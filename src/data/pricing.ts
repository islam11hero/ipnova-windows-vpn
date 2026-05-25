/** Reference pricing (EU market 2024–2025). */

export type PricingPlan = {
  id: string;
  name: string;
  durationLabel: string;
  priceEur: number;
  pricePerMonth: number;
  compareAtEur?: number;
  badge?: string;
  highlight?: boolean;
  features: string[];
  checkoutPath: string;
};

const siteBase = () =>
  (import.meta.env.VITE_SITE_URL ?? "http://127.0.0.1:3000").replace(
    /\/$/,
    "",
  );

export const PRICING_PLANS: PricingPlan[] = [
  {
    id: "monthly",
    name: "Monthly",
    durationLabel: "1 month",
    priceEur: 8.99,
    pricePerMonth: 8.99,
    features: ["Max speed", "Spain + more regions soon", "Secure data transfer"],
    checkoutPath: "/checkout?plan=monthly",
  },
  {
    id: "quarter",
    name: "3 months",
    durationLabel: "First payment — most popular",
    priceEur: 17.99,
    pricePerMonth: 6.0,
    compareAtEur: 26.97,
    badge: "Launch offer",
    highlight: true,
    features: [
      "Save 33% vs 3× monthly",
      "Spain server priority",
      "Priority support",
      "Unlimited data transfer*",
    ],
    checkoutPath: "/checkout?plan=quarter",
  },
  {
    id: "semiannual",
    name: "6 months",
    durationLabel: "6 months",
    priceEur: 29.99,
    pricePerMonth: 5.0,
    compareAtEur: 53.94,
    features: ["Best value for regular use", "All features included"],
    checkoutPath: "/checkout?plan=semiannual",
  },
  {
    id: "annual",
    name: "Annual",
    durationLabel: "12 months",
    priceEur: 49.99,
    pricePerMonth: 4.17,
    compareAtEur: 107.88,
    badge: "Best savings",
    features: ["Compare to ~€4/mo annual VPNs", "Optional auto-renewal"],
    checkoutPath: "/checkout?plan=annual",
  },
];

export function planCheckoutUrl(plan: PricingPlan): string {
  const base = `${siteBase()}${plan.checkoutPath}`;
  const sep = base.includes("?") ? "&" : "?";
  return `${base}${sep}source=windows_app`;
}

export const MARKET_NOTE =
  "Prices are indicative based on EU VPN market averages (2025). Final price on checkout.";

export const LAUNCH_OFFER = PRICING_PLANS.find((p) => p.id === "quarter")!;
