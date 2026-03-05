import { useState, useEffect } from 'react';
import { loadAIConfig, refreshSoul, refreshTools, refreshAll } from '../../../lib/ai/loader';
import type { AIConfig } from '../../../lib/ai/types';
import type { SoulConfig } from '../../../lib/ai/soul/types';
import type { ToolsConfig } from '../../../lib/ai/tools/types';
import SettingsHeader from '../components/SettingsHeader';
import { useSettingsNavigation } from '../hooks/useSettingsNavigation';

const AIPanel = () => {
  const { navigateBack } = useSettingsNavigation();
  const [aiConfig, setAiConfig] = useState<AIConfig | null>(null);
  const [loading, setLoading] = useState(false);
  const [refreshingComponent, setRefreshingComponent] = useState<'soul' | 'tools' | 'all' | null>(null);
  const [error, setError] = useState<string>('');

  useEffect(() => {
    loadAIPreview();
  }, []);

  const loadAIPreview = async () => {
    setLoading(true);
    setError('');
    try {
      const config = await loadAIConfig();
      setAiConfig(config);

      // Show metadata errors if any
      if (config.metadata.errors && config.metadata.errors.length > 0) {
        setError(config.metadata.errors.join('; '));
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to load AI configuration';
      setError(message);
    } finally {
      setLoading(false);
    }
  };

  const refreshSoulConfig = async () => {
    setRefreshingComponent('soul');
    setError('');
    try {
      const soulConfig = await refreshSoul();
      if (aiConfig) {
        setAiConfig({
          ...aiConfig,
          soul: soulConfig,
          metadata: {
            ...aiConfig.metadata,
            loadedAt: Date.now()
          }
        });
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to refresh SOUL configuration';
      setError(message);
    } finally {
      setRefreshingComponent(null);
    }
  };

  const refreshToolsConfig = async () => {
    setRefreshingComponent('tools');
    setError('');
    try {
      const toolsConfig = await refreshTools();
      if (aiConfig) {
        setAiConfig({
          ...aiConfig,
          tools: toolsConfig,
          metadata: {
            ...aiConfig.metadata,
            loadedAt: Date.now()
          }
        });
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to refresh TOOLS configuration';
      setError(message);
    } finally {
      setRefreshingComponent(null);
    }
  };

  const refreshAllConfig = async () => {
    setRefreshingComponent('all');
    setError('');
    try {
      const config = await refreshAll();
      setAiConfig(config);

      if (config.metadata.errors && config.metadata.errors.length > 0) {
        setError(config.metadata.errors.join('; '));
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to refresh AI configuration';
      setError(message);
    } finally {
      setRefreshingComponent(null);
    }
  };

  const formatPersonality = (config: SoulConfig): string => {
    return config.personality
      .slice(0, 3)
      .map(p => `${p.trait}: ${p.description}`)
      .join(' • ');
  };

  const formatSafetyRules = (config: SoulConfig): string => {
    return config.safetyRules
      .slice(0, 2)
      .map(r => r.rule)
      .join(' • ');
  };

  const formatToolsOverview = (config: ToolsConfig): string => {
    const skillNames = Object.keys(config.skillGroups);
    return skillNames
      .slice(0, 4)
      .map(skillId => {
        const group = config.skillGroups[skillId];
        return `${group.name} (${group.tools.length})`;
      })
      .join(' • ');
  };

  const formatCategories = (config: ToolsConfig): string => {
    return Object.values(config.categories)
      .filter(cat => cat.toolCount && cat.toolCount > 0)
      .slice(0, 3)
      .map(cat => `${cat.name}: ${cat.toolCount} tools`)
      .join(' • ');
  };

  return (
    <div className="h-full flex flex-col">
      <SettingsHeader title="AI Configuration" showBackButton={true} onBack={navigateBack} />

      <div className="flex-1 overflow-y-auto px-6 pb-10 space-y-6">
        {/* Overview Section */}
        <section className="space-y-4">
          <h3 className="text-lg font-semibold text-white">AI System Overview</h3>
          <p className="text-sm text-gray-400">
            AlphaHuman uses SOUL for persona configuration and TOOLS for external service integration.
          </p>

          {aiConfig && (
            <div className="bg-gray-900 rounded-lg p-4 border border-gray-700">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="text-xs text-gray-400 uppercase tracking-wide">Configuration Status</label>
                  <div className="text-sm text-green-400 font-medium mt-1">
                    {aiConfig.metadata.hasFallbacks ? 'Fallback Mode' : 'Fully Loaded'}
                  </div>
                </div>
                <div>
                  <label className="text-xs text-gray-400 uppercase tracking-wide">Loading Duration</label>
                  <div className="text-sm text-blue-400 font-medium mt-1">
                    {aiConfig.metadata.loadingDuration}ms
                  </div>
                </div>
              </div>
            </div>
          )}
        </section>

        {/* SOUL Configuration Section */}
        <section className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-semibold text-white">SOUL Persona Configuration</h3>
            <button
              onClick={refreshSoulConfig}
              className="text-sm text-blue-400 hover:text-blue-300 transition-colors disabled:opacity-50"
              disabled={refreshingComponent === 'soul'}
            >
              {refreshingComponent === 'soul' ? 'Refreshing...' : 'Refresh SOUL'}
            </button>
          </div>
          <p className="text-sm text-gray-400">
            The SOUL system injects persona context into every user message to ensure consistent AI behavior.
          </p>

          {loading && (
            <div className="text-sm text-gray-400 animate-pulse">Loading SOUL configuration...</div>
          )}

          {error && (
            <div className="bg-red-500/10 border border-red-500/40 rounded-lg p-3">
              <div className="text-sm text-red-200">{error}</div>
            </div>
          )}

          {aiConfig?.soul && (
            <div className="bg-gray-900 rounded-lg p-4 border border-gray-700 space-y-3">
              <div>
                <label className="text-xs text-gray-400 uppercase tracking-wide">Identity</label>
                <div className="text-sm text-green-400 font-medium mt-1">
                  {aiConfig.soul.identity.name}
                </div>
                <div className="text-xs text-gray-300 mt-1">
                  {aiConfig.soul.identity.description}
                </div>
              </div>

              {aiConfig.soul.personality.length > 0 && (
                <div>
                  <label className="text-xs text-gray-400 uppercase tracking-wide">Personality</label>
                  <div className="text-xs text-gray-300 mt-1 leading-relaxed">
                    {formatPersonality(aiConfig.soul)}
                  </div>
                </div>
              )}

              {aiConfig.soul.safetyRules.length > 0 && (
                <div>
                  <label className="text-xs text-gray-400 uppercase tracking-wide">Safety Rules</label>
                  <div className="text-xs text-yellow-300 mt-1 leading-relaxed">
                    {formatSafetyRules(aiConfig.soul)}
                  </div>
                </div>
              )}

              <div className="flex items-center justify-between pt-2 border-t border-gray-700">
                <div className="text-xs text-gray-400">
                  Source: {aiConfig.metadata.sources.soul}
                </div>
                <div className="text-xs text-gray-400">
                  Loaded: {new Date(aiConfig.soul.loadedAt).toLocaleTimeString()}
                </div>
              </div>
            </div>
          )}
        </section>

        {/* TOOLS Configuration Section */}
        <section className="space-y-4">
          <div className="flex items-center justify-between">
            <h3 className="text-lg font-semibold text-white">TOOLS Configuration</h3>
            <button
              onClick={refreshToolsConfig}
              className="text-sm text-blue-400 hover:text-blue-300 transition-colors disabled:opacity-50"
              disabled={refreshingComponent === 'tools'}
            >
              {refreshingComponent === 'tools' ? 'Refreshing...' : 'Refresh TOOLS'}
            </button>
          </div>
          <p className="text-sm text-gray-400">
            TOOLS provide AlphaHuman with the ability to interact with external services and perform actions.
          </p>

          {aiConfig?.tools && (
            <div className="bg-gray-900 rounded-lg p-4 border border-gray-700 space-y-3">
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="text-xs text-gray-400 uppercase tracking-wide">Tools Available</label>
                  <div className="text-sm text-green-400 font-medium mt-1">
                    {aiConfig.tools.statistics.totalTools} tools
                  </div>
                </div>
                <div>
                  <label className="text-xs text-gray-400 uppercase tracking-wide">Active Skills</label>
                  <div className="text-sm text-green-400 font-medium mt-1">
                    {aiConfig.tools.statistics.activeSkills} skills
                  </div>
                </div>
              </div>

              {Object.keys(aiConfig.tools.skillGroups).length > 0 && (
                <div>
                  <label className="text-xs text-gray-400 uppercase tracking-wide">Skills Overview</label>
                  <div className="text-xs text-gray-300 mt-1 leading-relaxed">
                    {formatToolsOverview(aiConfig.tools)}
                  </div>
                </div>
              )}

              {Object.keys(aiConfig.tools.categories).length > 0 && (
                <div>
                  <label className="text-xs text-gray-400 uppercase tracking-wide">Top Categories</label>
                  <div className="text-xs text-blue-300 mt-1 leading-relaxed">
                    {formatCategories(aiConfig.tools)}
                  </div>
                </div>
              )}

              <div className="flex items-center justify-between pt-2 border-t border-gray-700">
                <div className="text-xs text-gray-400">
                  Source: {aiConfig.metadata.sources.tools}
                </div>
                <div className="text-xs text-gray-400">
                  Loaded: {new Date(aiConfig.tools.loadedAt).toLocaleTimeString()}
                </div>
              </div>
            </div>
          )}
        </section>

        {/* Combined Actions */}
        <section className="space-y-4">
          <div className="flex items-center justify-center">
            <button
              onClick={refreshAllConfig}
              className="px-4 py-2 bg-blue-600 hover:bg-blue-700 text-white text-sm font-medium rounded-lg transition-colors disabled:opacity-50"
              disabled={refreshingComponent === 'all'}
            >
              {refreshingComponent === 'all' ? 'Refreshing All...' : 'Refresh All AI Configuration'}
            </button>
          </div>
        </section>
      </div>
    </div>
  );
};

export default AIPanel;