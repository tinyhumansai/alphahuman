/** SOUL persona configuration */
export interface SoulConfig {
  /** Raw markdown source */
  raw: string;
  /** Parsed persona identity */
  identity: SoulIdentity;
  /** Personality traits */
  personality: PersonalityTrait[];
  /** Voice and tone guidelines */
  voiceTone: VoiceToneGuideline[];
  /** Behavioral patterns */
  behaviors: BehaviorPattern[];
  /** Safety rules (never break) */
  safetyRules: SafetyRule[];
  /** Available games/interactions */
  interactions: Interaction[];
  /** Memory preferences */
  memorySettings: MemorySettings;
  /** Emergency responses */
  emergencyResponses: EmergencyResponse[];
  /** Whether this is the default soul or user-customized */
  isDefault: boolean;
  /** Last loaded timestamp */
  loadedAt: number;
}

export interface SoulIdentity {
  name: string;
  description: string;
}

export interface PersonalityTrait {
  trait: string;
  description: string;
}

export interface VoiceToneGuideline {
  guideline: string;
}

export interface BehaviorPattern {
  context: string;
  behaviors: string[];
}

export interface SafetyRule {
  id: string;
  rule: string;
  priority: number;
}

export interface Interaction {
  name: string;
  description: string;
}

export interface MemorySettings {
  remember: string[];
}

export interface EmergencyResponse {
  trigger: string;
  response: string;
}
