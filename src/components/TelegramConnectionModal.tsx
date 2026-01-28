import { useState, useEffect, useCallback, useRef } from 'react';
import { createPortal } from 'react-dom';
import { QRCodeSVG } from 'qrcode.react';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import {
  initializeTelegram,
  connectTelegram,
  checkAuthStatus,
  setAuthStatus,
  setAuthError,
  setConnectionStatus,
} from '../store/telegram';
import { selectIsInitialized, selectConnectionStatus } from '../store/telegramSelectors';
import { mtprotoService } from '../services/mtprotoService';

interface TelegramConnectionModalProps {
  isOpen: boolean;
  onClose: () => void;
  onComplete: () => void;
}

type ConnectionStep = 'qr' | '2fa' | 'loading' | 'error';

const TelegramConnectionModal = ({ isOpen, onClose, onComplete }: TelegramConnectionModalProps) => {
  const dispatch = useAppDispatch();
  const isInitialized = useAppSelector(selectIsInitialized);
  const connectionStatus = useAppSelector(selectConnectionStatus);

  const [currentStep, setCurrentStep] = useState<ConnectionStep>('qr');
  const [password, setPassword] = useState('');
  const [passwordHint, setPasswordHint] = useState<string | undefined>();
  const [qrCodeUrl, setQrCodeUrl] = useState<string | null>(null);
  const [qrCodeExpires, setQrCodeExpires] = useState<number | null>(null);
  const [timeRemaining, setTimeRemaining] = useState<number>(0);
  const [error, setError] = useState<string | null>(null);
  const [isAuthenticating, setIsAuthenticating] = useState(false);

  // Store password promise resolver
  const passwordResolverRef = useRef<((password: string) => void) | null>(null);

  // Initialize and connect when modal opens
  useEffect(() => {
    if (!isOpen) return;

    const init = async () => {
      try {
        setCurrentStep('loading');
        if (!isInitialized) {
          await dispatch(initializeTelegram()).unwrap();
        }
        if (connectionStatus !== 'connected') {
          await dispatch(connectTelegram()).unwrap();
        }
        // Check if already authenticated (but don't fail if not authenticated)
        try {
          const authCheck = await dispatch(checkAuthStatus()).unwrap();
          if (authCheck) {
            onComplete();
            onClose();
            return;
          }
        } catch (err) {
          // If checkAuthStatus fails, we're not authenticated - continue with QR flow
          console.log('Not authenticated, proceeding with QR code flow');
        }
        // Start QR code flow
        startQrCodeFlow();
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Failed to initialize Telegram';
        setError(errorMessage);
        setCurrentStep('error');
        dispatch(setConnectionStatus('error'));
      }
    };

    init();
  }, [isOpen, isInitialized, connectionStatus, dispatch, onComplete, onClose]);

  const startQrCodeFlow = useCallback(async () => {
    try {
      setIsAuthenticating(true);
      setError(null);
      setCurrentStep('qr');
      setPassword('');
      setPasswordHint(undefined);
      setQrCodeUrl(null);
      setQrCodeExpires(null);

      await mtprotoService.signInWithQrCode(
        (qrCode) => {
          // Convert Buffer/Uint8Array to base64url for QR code URL
          let tokenBase64: string;
          if (qrCode.token instanceof Uint8Array) {
            // Convert Uint8Array to base64url
            const binary = Array.from(qrCode.token)
              .map((byte) => String.fromCharCode(byte))
              .join('');
            tokenBase64 = btoa(binary)
              .replace(/\+/g, '-')
              .replace(/\//g, '_')
              .replace(/=/g, '');
          } else {
            // If it's a Buffer, use toString
            const buffer = qrCode.token as { toString: (encoding: string) => string };
            tokenBase64 = buffer.toString('base64url');
          }
          const url = `tg://login?token=${tokenBase64}`;
          setQrCodeUrl(url);
          setQrCodeExpires(qrCode.expires);
        },
        async (hint) => {
          // 2FA password required
          console.log('Password callback invoked with hint:', hint);
          setPasswordHint(hint);
          setCurrentStep('2fa');
          setIsAuthenticating(false);

          // Wait for user to enter password
          return new Promise<string>((resolve, reject) => {
            passwordResolverRef.current = (password: string) => {
              console.log('Password resolved in callback, sending to Telegram:', password ? '***' : 'empty');
              if (!password || !password.trim()) {
                reject(new Error('Password is required'));
                return;
              }
              resolve(password.trim());
            };

            // Set a timeout to prevent hanging forever
            setTimeout(() => {
              if (passwordResolverRef.current) {
                console.warn('Password callback timeout - no password provided');
                reject(new Error('Password input timeout'));
                passwordResolverRef.current = null;
              }
            }, 300000); // 5 minutes timeout
          });
        },
        async (err) => {
          // Handle errors
          const errorMessage = err.message || 'Authentication error';

          // Check if it's a 2FA password needed error
          if (errorMessage.includes('SESSION_PASSWORD_NEEDED') || errorMessage.includes('PASSWORD')) {
            // This should trigger the password callback, but if it doesn't, handle it here
            setPasswordHint(undefined);
            setCurrentStep('2fa');
            setIsAuthenticating(false);
            // Don't stop authentication - let the password callback handle it
            return false;
          }

          setError(errorMessage);
          dispatch(setAuthError(errorMessage));

          // Check if it's a cancellation
          if (errorMessage.includes('AUTH_USER_CANCEL') || errorMessage.includes('cancel')) {
            setCurrentStep('qr');
            setIsAuthenticating(false);
            return true; // Stop authentication
          }

          return false; // Continue
        }
      );

      // Authentication successful
      setIsAuthenticating(false);
      try {
        await dispatch(checkAuthStatus()).unwrap();
      } catch (err) {
        // Even if checkAuthStatus fails, we know we just authenticated
        console.warn('checkAuthStatus failed after authentication, but continuing:', err);
      }
      dispatch(setAuthStatus('authenticated'));
      onComplete();
      onClose();
    } catch (err) {
      setIsAuthenticating(false);
      const errorMessage = err instanceof Error ? err.message : 'Authentication failed';

      // If it's a password needed error, we should already be on the 2FA step
      // Don't show it as an error, just log it
      if (errorMessage.includes('SESSION_PASSWORD_NEEDED')) {
        console.log('Password required - user should be on 2FA step');
        // If we're not on 2FA step, switch to it
        if (currentStep !== '2fa') {
          setCurrentStep('2fa');
          setPasswordHint(undefined);
        }
        return; // Don't show error, password callback should handle it
      }

      setError(errorMessage);
      setCurrentStep('error');
      dispatch(setAuthError(errorMessage));
    }
  }, [dispatch, onComplete, onClose]);

  // Update countdown timer every second and reload QR code on timeout
  useEffect(() => {
    if (!qrCodeExpires) {
      setTimeRemaining(0);
      return;
    }

    const updateTimer = () => {
      const remaining = Math.max(0, Math.floor((qrCodeExpires * 1000 - Date.now()) / 1000));
      setTimeRemaining(remaining);

      // If timer reaches 0 and we're on QR step, reload the QR code
      if (remaining === 0 && currentStep === 'qr' && !isAuthenticating) {
        startQrCodeFlow();
      }
    };

    // Update immediately
    updateTimer();

    // Update every second
    const interval = setInterval(updateTimer, 1000);

    return () => clearInterval(interval);
  }, [qrCodeExpires, currentStep, isAuthenticating, startQrCodeFlow]);

  // Poll authentication status every 5 seconds when QR code is displayed
  useEffect(() => {
    if (currentStep !== 'qr' || !qrCodeUrl || isAuthenticating) {
      return;
    }

    const checkAuthStatus = async () => {
      try {
        const client = mtprotoService.getClient();
        if (!client) return;

        const isAuthorized = await client.checkAuthorization();

        if (isAuthorized) {
          // User is authenticated - check if we need password
          try {
            // Try to get user info - if this fails with SESSION_PASSWORD_NEEDED, we need password
            await client.getMe();
            // If getMe succeeds, authentication is complete
            console.log('QR code scanned and authenticated successfully');
            // The signInWithQrCode promise should resolve, but if it doesn't, trigger completion
            setIsAuthenticating(false);
            dispatch(setAuthStatus('authenticated'));
            onComplete();
            onClose();
          } catch (err) {
            const errorMessage = err instanceof Error ? err.message : String(err);
            if (errorMessage.includes('SESSION_PASSWORD_NEEDED') || errorMessage.includes('PASSWORD')) {
              // Need password - switch to 2FA step
              console.log('Password required after QR scan - switching to 2FA step');
              if (currentStep === 'qr' && !passwordResolverRef.current) {
                // Trigger password callback manually if it wasn't called
                setPasswordHint(undefined);
                setCurrentStep('2fa');
                setIsAuthenticating(false);
              }
            }
          }
        }
      } catch (err) {
        // Ignore errors during polling - they're expected if not authenticated yet
        console.debug('Auth status check:', err instanceof Error ? err.message : String(err));
      }
    };

    // Check immediately, then every 5 seconds
    checkAuthStatus();
    const interval = setInterval(checkAuthStatus, 5000);

    return () => clearInterval(interval);
  }, [currentStep, qrCodeUrl, isAuthenticating, dispatch, onComplete, onClose]);

  const handle2FASubmit = async () => {
    const passwordValue = password.trim();

    if (!passwordValue) {
      setError('Please enter your password');
      return;
    }

    if (!passwordResolverRef.current) {
      console.error('Password resolver is null - password callback may not be active');
      setError('Authentication session expired. Please try again.');
      setCurrentStep('qr');
      return;
    }

    try {
      setIsAuthenticating(true);
      setError(null);

      // Resolve the password promise to continue authentication
      // This will pass the password to the Telegram library
      const resolver = passwordResolverRef.current;
      passwordResolverRef.current = null; // Clear before resolving to prevent double calls

      console.log('Resolving password promise - sending password to Telegram');
      resolver(passwordValue);

      // The authentication will continue in the background via signInWithQrCode
      // The success/error will be handled by the signInWithQrCode promise
      // We keep isAuthenticating true until the promise resolves or rejects
    } catch (err) {
      setIsAuthenticating(false);
      const errorMessage = err instanceof Error ? err.message : 'Password verification failed';
      setError(errorMessage);
      dispatch(setAuthError(errorMessage));
      console.error('Error in handle2FASubmit:', err);
    }
  };

  const handleBack = () => {
    if (currentStep === '2fa') {
      setCurrentStep('qr');
      setPassword('');
      setPasswordHint(undefined);
      if (passwordResolverRef.current) {
        passwordResolverRef.current('');
        passwordResolverRef.current = null;
      }
    } else {
      onClose();
    }
  };

  const handleRetry = () => {
    setError(null);
    setPassword('');
    setPasswordHint(undefined);
    setQrCodeUrl(null);
    setQrCodeExpires(null);
    startQrCodeFlow();
  };

  if (!isOpen) return null;

  const modalContent = (
    <div
      className="fixed inset-0 z-[9999] bg-black/80 backdrop-blur-sm flex items-center justify-center"
      style={{
        position: 'fixed',
        top: 0,
        left: 0,
        right: 0,
        bottom: 0,
        width: '100vw',
        height: '100vh',
        zIndex: 9999
      }}
    >
      <div
        className="bg-black/90 shadow-large animate-fade-up max-w-4xl max-h-[90vh] overflow-y-auto flex flex-col items-center justify-center rounded-3xl focus:outline-none"
        style={{
          maxWidth: '56rem',
          maxHeight: '90vh',
          padding: 0
        }}
      >
        {/* Close button */}
        <button
          onClick={onClose}
          className="absolute top-6 right-6 z-10 w-8 h-8 flex items-center justify-center rounded-full hover:bg-stone-800/50 transition-colors"
        >
          <svg className="w-5 h-5 opacity-70" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>

        {currentStep === 'loading' ? (
          <div className="text-center py-8 flex flex-col items-center justify-center">
            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-500 mb-4"></div>
            <p className="opacity-70">Initializing Telegram connection...</p>
          </div>
        ) : currentStep === 'error' ? (
          <>
            {/* Error Screen */}
            <div className="text-center flex flex-col items-center justify-center">
              <div className="w-16 h-16 bg-red-500/20 rounded-full flex items-center justify-center mx-auto mb-4">
                <svg className="w-8 h-8 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </div>
              <h2 className="text-2xl font-bold mb-2">Connection Error</h2>
              <p className="opacity-70 text-sm mb-6">{error || 'An error occurred'}</p>
              <div className="flex space-x-3">
                <button
                  onClick={handleBack}
                  className="flex-1 py-2.5 px-4 bg-stone-800/50 hover:bg-stone-700/50 border border-stone-700 rounded-xl text-sm font-medium transition-all duration-200"
                >
                  Cancel
                </button>
                <button
                  onClick={handleRetry}
                  className="flex-1 py-2.5 px-4 bg-primary-500 hover:bg-primary-600 active:bg-primary-700 text-white rounded-xl text-sm font-medium transition-all duration-200"
                >
                  Retry
                </button>
              </div>
            </div>
          </>
        ) : currentStep === 'qr' ? (
          <>
            {/* QR Code Screen */}
            <div className="text-center flex flex-col items-center justify-center w-full">
              {/* QR Code Container */}
              <div className="flex justify-center mb-8">
                <div className="bg-white p-4 rounded-2xl shadow-large">
                  {qrCodeUrl ? (
                    <div className="relative w-64 h-64 flex items-center justify-center">
                      <QRCodeSVG
                        value={qrCodeUrl}
                        size={256}
                        level="H"
                        includeMargin={true}
                        marginSize={1}
                        bgColor="#FFFFFF"
                        fgColor="#000000"
                        className="w-full h-full"
                      />
                    </div>
                  ) : (
                    <div className="w-64 h-64 bg-gray-100 rounded-xl flex items-center justify-center">
                      {isAuthenticating ? (
                        <div className="text-center">
                          <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-500 mx-auto mb-4"></div>
                          <p className="text-gray-600 text-sm">Generating QR code...</p>
                        </div>
                      ) : (
                        <p className="text-gray-600 text-sm">Loading QR code...</p>
                      )}
                    </div>
                  )}
                </div>
              </div>

              {qrCodeExpires && timeRemaining > 0 && (
                <p className="text-xs opacity-70 mb-4">
                  This code expires in {timeRemaining} seconds
                </p>
              )}

              {error && (
                <div className="mb-4 p-3 bg-red-500/20 border border-red-500/50 rounded-xl">
                  <p className="text-red-400 text-sm">{error}</p>
                </div>
              )}

              {/* Instructions */}
              <div className="space-y-4 mb-6">
                <div className="flex items-start space-x-3 text-left">
                  <div className="w-6 h-6 bg-purple-500 rounded-full flex items-center justify-center flex-shrink-0 mt-0.5">
                    <span className="text-white font-bold text-xs">1</span>
                  </div>
                  <p className="opacity-90 text-sm">Open Telegram on your phone</p>
                </div>

                <div className="flex items-start space-x-3 text-left">
                  <div className="w-6 h-6 bg-purple-500 rounded-full flex items-center justify-center flex-shrink-0 mt-0.5">
                    <span className="text-white font-bold text-xs">2</span>
                  </div>
                  <p className="opacity-90 text-sm">Go to Settings &gt; Devices &gt; Link Desktop Device</p>
                </div>

                <div className="flex items-start space-x-3 text-left">
                  <div className="w-6 h-6 bg-purple-500 rounded-full flex items-center justify-center flex-shrink-0 mt-0.5">
                    <span className="text-white font-bold text-xs">3</span>
                  </div>
                  <p className="opacity-90 text-sm">Point your phone at this screen to confirm login</p>
                </div>
              </div>
            </div>
          </>
        ) : (
          <>
            {/* 2FA Screen */}
            <div className="text-center flex flex-col items-center justify-center w-full">
              <h2 className="text-2xl font-bold mb-2">Enter Your Password</h2>
              <p className="opacity-70 text-sm mb-6">
                {passwordHint
                  ? `Your account is protected with two-step verification. Hint: ${passwordHint}`
                  : 'Your account is protected with two-step verification. Please enter your password to continue.'}
              </p>

              {error && (
                <div className="mb-4 p-3 bg-red-500/20 border border-red-500/50 rounded-xl">
                  <p className="text-red-400 text-sm">{error}</p>
                </div>
              )}

              {/* Password input */}
              <div className="mb-6">
                <input
                  type="password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' && password.trim() && !isAuthenticating) {
                      handle2FASubmit();
                    }
                  }}
                  placeholder="Enter your password"
                  className="w-full px-4 py-3 bg-black/50 border border-stone-700 rounded-xl text-white placeholder-opacity-50 focus:outline-none focus:border-primary-500 transition-colors"
                  autoComplete="off"
                  data-form-type="other"
                  data-lpignore="true"
                  data-1p-ignore="true"
                  data-bwignore="true"
                  data-dashlane-ignore="true"
                  data-bitwarden-watching="false"
                  autoFocus
                  disabled={isAuthenticating}
                />
              </div>

              {/* Action buttons */}
              <div className="flex space-x-3">
                <button
                  onClick={handleBack}
                  disabled={isAuthenticating}
                  className="flex-1 py-2.5 px-4 bg-stone-800/50 hover:bg-stone-700/50 border border-stone-700 rounded-xl text-sm font-medium transition-all duration-200 disabled:opacity-50"
                >
                  Back
                </button>
                <button
                  onClick={handle2FASubmit}
                  disabled={!password.trim() || isAuthenticating}
                  className="flex-1 py-2.5 px-4 bg-primary-500 hover:bg-primary-600 active:bg-primary-700 disabled:opacity-50 disabled:cursor-not-allowed text-white rounded-xl text-sm font-medium transition-all duration-200"
                >
                  {isAuthenticating ? 'Verifying...' : 'Continue'}
                </button>
              </div>
            </div>
          </>
        )}
      </div>
    </div>
  );

  return createPortal(modalContent, document.body);
};

export default TelegramConnectionModal;
