export type Country = {
  id: string;
  name: string;
  city: string;
  flag: string;
  available: boolean;
  ping?: number;
};

export const DEFAULT_COUNTRY_ID = "es";

export const COUNTRIES: Country[] = [
  {
    id: "es",
    name: "Spain",
    city: "Madrid",
    flag: "🇪🇸",
    available: true,
    ping: 24,
  },
  {
    id: "fr",
    name: "France",
    city: "Paris",
    flag: "🇫🇷",
    available: false,
    ping: 32,
  },
  {
    id: "de",
    name: "Germany",
    city: "Frankfurt",
    flag: "🇩🇪",
    available: false,
    ping: 28,
  },
  {
    id: "nl",
    name: "Netherlands",
    city: "Amsterdam",
    flag: "🇳🇱",
    available: false,
    ping: 30,
  },
  {
    id: "us",
    name: "United States",
    city: "New York",
    flag: "🇺🇸",
    available: false,
    ping: 89,
  },
];

export function getCountry(id: string): Country | undefined {
  return COUNTRIES.find((c) => c.id === id);
}
