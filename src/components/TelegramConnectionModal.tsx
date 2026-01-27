import { useState, useEffect, useCallback, useRef } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import {
  initializeTelegram,
  connectTelegram,
  checkAuthStatus,
  setAuthStatus,
  setAuthError,
  setConnectionStatus,
} from '../store/telegramSlice';
import { selectIsInitialized, selectConnectionStatus, selectAuthStatus } from '../store/telegramSelectors';
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
  const authStatus = useAppSelector(selectAuthStatus);

  const [currentStep, setCurrentStep] = useState<ConnectionStep>('qr');
  const [password, setPassword] = useState('');
  const [passwordHint, setPasswordHint] = useState<string | undefined>();
  const [qrCodeUrl, setQrCodeUrl] = useState<string | null>(null);
  const [qrCodeExpires, setQrCodeExpires] = useState<number | null>(null);
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
        // Check if already authenticated
        const authCheck = await dispatch(checkAuthStatus()).unwrap();
        if (authCheck) {
          onComplete();
          onClose();
          return;
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
          setPasswordHint(hint);
          setCurrentStep('2fa');
          setIsAuthenticating(false);
          
          // Wait for user to enter password
          return new Promise<string>((resolve) => {
            passwordResolverRef.current = resolve;
          });
        },
        async (err) => {
          // Handle errors
          const errorMessage = err.message || 'Authentication error';
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
      await dispatch(checkAuthStatus()).unwrap();
      dispatch(setAuthStatus('authenticated'));
      onComplete();
      onClose();
    } catch (err) {
      setIsAuthenticating(false);
      const errorMessage = err instanceof Error ? err.message : 'Authentication failed';
      setError(errorMessage);
      setCurrentStep('error');
      dispatch(setAuthError(errorMessage));
    }
  }, [dispatch, onComplete, onClose]);

  const handle2FASubmit = async () => {
    if (!password.trim() || !passwordResolverRef.current) return;

    try {
      setIsAuthenticating(true);
      setError(null);
      
      // Resolve the password promise to continue authentication
      passwordResolverRef.current(password);
      passwordResolverRef.current = null;
    } catch (err) {
      setIsAuthenticating(false);
      const errorMessage = err instanceof Error ? err.message : 'Password verification failed';
      setError(errorMessage);
      dispatch(setAuthError(errorMessage));
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

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm">
      <div className="relative w-full max-w-md mx-4">
        <div className="glass rounded-3xl p-8 shadow-large animate-fade-up">
          {/* Close button */}
          <button
            onClick={onClose}
            className="absolute top-4 right-4 w-8 h-8 flex items-center justify-center rounded-full hover:bg-stone-800/50 transition-colors"
          >
            <svg className="w-5 h-5 opacity-70" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>

          {currentStep === 'loading' ? (
            <div className="text-center py-8">
              <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-primary-500 mx-auto mb-4"></div>
              <p className="opacity-70">Initializing Telegram connection...</p>
            </div>
          ) : currentStep === 'error' ? (
            <>
              {/* Error Screen */}
              <div className="text-center">
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
              <div className="text-center">
                <h2 className="text-2xl font-bold mb-6">Log in to Telegram by QR Code</h2>

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
                        {/* Telegram logo overlay */}
                        <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
                          <div className="w-16 h-16 bg-[#0088CC] rounded-full flex items-center justify-center shadow-lg">
                            <svg className="w-10 h-10 text-white" fill="currentColor" viewBox="0 0 24 24">
                              <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
                            </svg>
                          </div>
                        </div>
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

                {qrCodeExpires && (
                  <p className="text-xs opacity-70 mb-4">
                    This code expires in {Math.max(0, Math.floor((qrCodeExpires * 1000 - Date.now()) / 1000))} seconds
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

                {/* Action buttons */}
                <div className="flex space-x-3">
                  <button
                    onClick={handleBack}
                    className="flex-1 py-2.5 px-4 bg-stone-800/50 hover:bg-stone-700/50 border border-stone-700 rounded-xl text-sm font-medium transition-all duration-200"
                  >
                    Cancel
                  </button>
                  <button
                    onClick={handleRetry}
                    disabled={isAuthenticating}
                    className="flex-1 py-2.5 px-4 bg-primary-500 hover:bg-primary-600 active:bg-primary-700 disabled:opacity-50 disabled:cursor-not-allowed text-white rounded-xl text-sm font-medium transition-all duration-200"
                  >
                    {isAuthenticating ? 'Connecting...' : 'Refresh QR Code'}
                  </button>
                </div>
              </div>
            </>
          ) : (
            <>
              {/* 2FA Screen */}
              <div className="text-center">
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
    </div>
  );
};

export default TelegramConnectionModal;
