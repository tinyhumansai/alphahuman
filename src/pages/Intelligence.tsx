export default function Intelligence() {
  return (
    <div className="min-h-full relative">
      <div className="relative z-10 min-h-full flex flex-col items-center justify-center">
        <div className="max-w-md mx-auto text-center px-6">
          {/* Icon */}
          <div className="w-16 h-16 mx-auto mb-6 rounded-2xl bg-white/[0.05] border border-white/[0.08] flex items-center justify-center">
            <svg className="w-8 h-8 text-primary-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={1.5}
                d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z"
              />
            </svg>
          </div>

          <h1 className="text-2xl font-bold text-white mb-2">Intelligence</h1>
          <p className="text-stone-400 text-sm mb-6">
            AI-powered insights, memory, and analytics are coming soon.
          </p>

          <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-primary-500/10 border border-primary-500/20">
            <div className="w-2 h-2 rounded-full bg-primary-400 animate-pulse" />
            <span className="text-xs font-medium text-primary-400">Coming Soon</span>
          </div>
        </div>
      </div>
    </div>
  );
}
