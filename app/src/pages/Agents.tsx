const Agents = () => {
  return (
    <div className="min-h-full relative">
      <div className="relative z-10 min-h-full flex flex-col">
        <div className="flex-1 flex items-center justify-center p-4">
          <div className="max-w-md w-full">
            <div className="glass rounded-3xl p-8 shadow-large animate-fade-up text-center">
              <div className="flex justify-center mb-4">
                <svg
                  className="w-12 h-12 text-primary-400 opacity-60"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24">
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={1.5}
                    d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z"
                  />
                </svg>
              </div>
              <h1 className="text-xl font-bold mb-2">Agents</h1>
              <p className="text-sm opacity-60">Your AI agents will appear here</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default Agents;
