import { type KeyboardEvent, useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useNavigate } from 'react-router-dom';

import LottieAnimation from '../components/LottieAnimation';
import { skillManager } from '../lib/skills/manager';
import { setEncryptionKeyForUser } from '../store/authSlice';
import { useAppDispatch, useAppSelector } from '../store/hooks';
import {
  deriveAesKeyFromMnemonic,
  deriveEvmAddressFromMnemonic,
  generateMnemonicPhrase,
  validateMnemonicPhrase,
} from '../utils/cryptoKeys';

const WORD_COUNT = 24;

const Mnemonic = () => {
  const navigate = useNavigate();
  const dispatch = useAppDispatch();
  const user = useAppSelector(state => state.user.user);
  const [mode, setMode] = useState<'generate' | 'import'>('generate');
  const [copied, setCopied] = useState(false);
  const [confirmed, setConfirmed] = useState(false);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Generate mode state
  const mnemonic = useMemo(() => generateMnemonicPhrase(), []);
  const words = useMemo(() => mnemonic.split(' '), [mnemonic]);

  // Import mode state
  const [importWords, setImportWords] = useState<string[]>(Array(WORD_COUNT).fill(''));
  const [importValid, setImportValid] = useState<boolean | null>(null);
  const inputRefs = useRef<(HTMLInputElement | null)[]>([]);

  useEffect(() => {
    if (copied) {
      const timer = setTimeout(() => setCopied(false), 3000);
      return () => clearTimeout(timer);
    }
  }, [copied]);

  // Reset state when switching modes
  useEffect(() => {
    setConfirmed(false);
    setError(null);
    setImportValid(null);
  }, [mode]);

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(mnemonic);
      setCopied(true);
    } catch {
      const textarea = document.createElement('textarea');
      textarea.value = mnemonic;
      textarea.style.position = 'fixed';
      textarea.style.opacity = '0';
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand('copy');
      document.body.removeChild(textarea);
      setCopied(true);
    }
  }, [mnemonic]);

  const handleImportWordChange = useCallback(
    (index: number, value: string) => {
      // Handle paste of full mnemonic phrase
      const pastedWords = value.trim().split(/\s+/);
      if (pastedWords.length > 1) {
        const newWords = [...importWords];
        for (let i = 0; i < Math.min(pastedWords.length, WORD_COUNT - index); i++) {
          newWords[index + i] = pastedWords[i].toLowerCase();
        }
        setImportWords(newWords);
        setImportValid(null);
        // Focus the next empty field or the last filled field
        const nextEmpty = newWords.findIndex(w => !w);
        const focusIndex = nextEmpty === -1 ? WORD_COUNT - 1 : nextEmpty;
        inputRefs.current[focusIndex]?.focus();
        return;
      }

      const newWords = [...importWords];
      newWords[index] = value.toLowerCase().trim();
      setImportWords(newWords);
      setImportValid(null);

      // Auto-advance to next input when a word is entered
      if (value.trim() && index < WORD_COUNT - 1) {
        inputRefs.current[index + 1]?.focus();
      }
    },
    [importWords]
  );

  const handleImportKeyDown = useCallback(
    (index: number, e: KeyboardEvent<HTMLInputElement>) => {
      if (e.key === 'Backspace' && !importWords[index] && index > 0) {
        inputRefs.current[index - 1]?.focus();
      }
    },
    [importWords]
  );

  const handleValidateImport = useCallback(() => {
    const phrase = importWords.join(' ').trim();
    const filledWords = importWords.filter(w => w.trim());

    if (filledWords.length !== WORD_COUNT) {
      setError(`Please enter all ${WORD_COUNT} words.`);
      setImportValid(false);
      return false;
    }

    const isValid = validateMnemonicPhrase(phrase);
    setImportValid(isValid);

    if (!isValid) {
      setError('Invalid recovery phrase. Please check your words and try again.');
      return false;
    }

    setError(null);
    return true;
  }, [importWords]);

  const handleContinue = async () => {
    setError(null);
    setLoading(true);

    try {
      let phraseToUse: string;

      if (mode === 'import') {
        if (!handleValidateImport()) {
          setLoading(false);
          return;
        }
        phraseToUse = importWords.join(' ').trim();
      } else {
        if (!confirmed) {
          setLoading(false);
          return;
        }
        phraseToUse = mnemonic;
      }

      const aesKey = deriveAesKeyFromMnemonic(phraseToUse);
      const walletAddress = deriveEvmAddressFromMnemonic(phraseToUse);

      if (!user?._id) {
        const msg = 'User not loaded. Please sign in again or refresh the page.';
        setError(msg);
        console.error('[Mnemonic] Cannot save encryption key: user not loaded');
        return;
      }
      dispatch(setEncryptionKeyForUser({ userId: user._id, key: aesKey }));
      await skillManager.setWalletAddress(walletAddress);
      navigate('/home');
    } catch (e) {
      setError(e instanceof Error ? e.message : 'Something went wrong. Please try again.');
    } finally {
      setLoading(false);
    }
  };

  const isImportComplete = importWords.every(w => w.trim());
  const canContinue = mode === 'generate' ? confirmed : isImportComplete;

  return (
    <div className="min-h-full relative flex items-center justify-center">
      <div className="relative z-10 max-w-lg w-full mx-4">
        <div className="flex justify-center mb-6">
          <LottieAnimation src="/lottie/safe3.json" height={120} width={120} />
        </div>

        <div className="glass rounded-3xl p-8 shadow-large animate-fade-up">
          {mode === 'generate' ? (
            <>
              <div className="text-center mb-4">
                <h1 className="text-xl font-bold mb-2">Your Recovery Phrase</h1>
                <p className="opacity-70 text-sm">
                  Write down these 24 words in order and store them somewhere safe. This phrase is
                  used to encrypt your data and can never be recovered if lost.
                </p>
              </div>

              {/* Mnemonic Grid */}
              <div className="bg-stone-900/5 rounded-2xl p-4 mb-4">
                <div className="grid grid-cols-3 gap-2">
                  {words.map((word, index) => (
                    <div
                      key={index}
                      className="flex items-center gap-2 bg-white/60 rounded-lg px-3 py-2 text-sm">
                      <span className="text-stone-400 font-mono text-xs w-5 text-right">
                        {index + 1}.
                      </span>
                      <span className="font-mono font-medium text-stone-800">{word}</span>
                    </div>
                  ))}
                </div>
              </div>

              {/* Copy Button */}
              <button
                onClick={handleCopy}
                className="w-full flex items-center justify-center gap-2 border border-stone-200 hover:bg-stone-50 text-stone-700 font-medium py-2.5 text-sm rounded-xl transition-all duration-200 mb-3">
                {copied ? (
                  <>
                    <svg
                      className="w-4 h-4 text-sage-500"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                      strokeWidth={2}>
                      <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
                    </svg>
                    <span className="text-sage-600">Copied to Clipboard</span>
                  </>
                ) : (
                  <>
                    <svg
                      className="w-4 h-4"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                      strokeWidth={2}>
                      <path
                        strokeLinecap="round"
                        strokeLinejoin="round"
                        d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z"
                      />
                    </svg>
                    <span>Copy to Clipboard</span>
                  </>
                )}
              </button>

              {/* Import existing link */}
              <button
                onClick={() => setMode('import')}
                className="w-full text-center text-sm text-primary-500 hover:text-primary-600 transition-colors mb-3">
                I already have a recovery phrase
              </button>

              {/* Warning */}
              <div className="flex items-start gap-3 bg-amber-50 border border-amber-200 rounded-xl p-3 mb-4">
                <svg
                  className="w-5 h-5 text-amber-500 shrink-0 mt-0.5"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                  strokeWidth={2}>
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z"
                  />
                </svg>
                <p className="text-xs text-amber-800">
                  Never share your recovery phrase with anyone. Anyone with these words can access
                  your encrypted data. OpenHuman will never ask for your recovery phrase.
                </p>
              </div>

              {/* Confirmation Checkbox */}
              <label className="flex items-start gap-3 cursor-pointer mb-4">
                <input
                  type="checkbox"
                  checked={confirmed}
                  onChange={e => setConfirmed(e.target.checked)}
                  className="mt-0.5 w-4 h-4 rounded border-stone-300 text-primary-500 focus:ring-primary-500"
                />
                <span className="text-sm text-stone-600">
                  I have saved my recovery phrase in a safe place
                </span>
              </label>
            </>
          ) : (
            <>
              <div className="text-center mb-4">
                <h1 className="text-xl font-bold mb-2">Import Recovery Phrase</h1>
                <p className="opacity-70 text-sm">
                  Enter your existing 24-word recovery phrase below. You can also paste the full
                  phrase into the first field.
                </p>
              </div>

              {/* Import Word Inputs Grid */}
              <div className="bg-stone-900/5 rounded-2xl p-4 mb-4">
                <div className="grid grid-cols-3 gap-2">
                  {importWords.map((word, index) => (
                    <div key={index} className="flex items-center gap-1.5">
                      <span className="text-stone-400 font-mono text-xs w-5 text-right shrink-0">
                        {index + 1}.
                      </span>
                      <input
                        ref={el => {
                          inputRefs.current[index] = el;
                        }}
                        type="text"
                        value={word}
                        onChange={e => handleImportWordChange(index, e.target.value)}
                        onKeyDown={e => handleImportKeyDown(index, e)}
                        autoComplete="off"
                        spellCheck={false}
                        className={`w-full font-mono text-sm font-medium px-2 py-1.5 rounded-lg border bg-white/60 outline-none transition-colors ${
                          importValid === false && word.trim()
                            ? 'border-coral-300 focus:border-coral-400'
                            : importValid === true
                              ? 'border-sage-300 focus:border-sage-400'
                              : 'border-stone-200 focus:border-primary-400'
                        }`}
                      />
                    </div>
                  ))}
                </div>
              </div>

              {/* Validation status */}
              {importValid === true && (
                <div className="flex items-center gap-2 text-sage-600 text-sm mb-3 justify-center">
                  <svg
                    className="w-4 h-4"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke="currentColor"
                    strokeWidth={2}>
                    <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
                  </svg>
                  <span>Valid recovery phrase</span>
                </div>
              )}

              {/* Back to generate link */}
              <button
                onClick={() => setMode('generate')}
                className="w-full text-center text-sm text-primary-500 hover:text-primary-600 transition-colors mb-3">
                Generate a new recovery phrase instead
              </button>
            </>
          )}

          {error && <p className="text-coral-500 text-sm mb-3 text-center">{error}</p>}

          {/* Continue Button */}
          <button
            onClick={handleContinue}
            disabled={!canContinue || loading}
            className="w-full flex items-center justify-center space-x-3 bg-blue-500 hover:bg-blue-600 active:bg-blue-700 disabled:opacity-60 disabled:cursor-not-allowed text-white font-semibold py-2.5 text-sm rounded-xl transition-all duration-300 hover:shadow-medium">
            <span>
              {loading
                ? 'Securing Your Data...'
                : mode === 'import'
                  ? 'Import & Continue'
                  : "I'm Ready! Let's Go!"}
            </span>
          </button>
        </div>
      </div>
    </div>
  );
};

export default Mnemonic;
