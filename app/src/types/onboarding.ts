// Onboarding flow types and interfaces

export interface OnboardingStep {
  id: string;
  title: string;
  description: string;
  component: React.ComponentType;
}

export interface UserProfile {
  name?: string;
  phone?: string;
  countryCode?: string;
  privacySettings?: PrivacySettings;
  analyticsPreferences?: AnalyticsPreferences;
}

export interface PrivacySettings {
  enterpriseGradeSecurity: boolean;
  dataEncryption: boolean;
  secureBackups: boolean;
}

export interface AnalyticsPreferences {
  shareAnalytics: boolean;
  emailConnected: boolean;
  maximumPrivacy: boolean;
}

export interface WeatherData {
  location: string;
  temperature: number;
  condition: string;
  icon: string;
}

export interface Country {
  code: string;
  name: string;
  flag: string;
  dialCode: string;
}
