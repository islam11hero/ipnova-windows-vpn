import { Gauge, Globe2, Settings, ShoppingBag } from "lucide-react";

import { LAUNCH_OFFER } from "../data/pricing";

export type NavId = "home" | "servers" | "plans" | "settings";

type NavItem = {
  id: NavId;
  label: string;
  icon: React.ReactNode;
};

const NAV: NavItem[] = [
  { id: "home", label: "Home", icon: <Gauge size={20} strokeWidth={1.75} /> },
  { id: "servers", label: "Servers", icon: <Globe2 size={20} strokeWidth={1.75} /> },
  { id: "plans", label: "Plans", icon: <ShoppingBag size={20} strokeWidth={1.75} /> },
  { id: "settings", label: "Settings", icon: <Settings size={20} strokeWidth={1.75} /> },
];

type Props = {
  active: NavId;
  onNavigate: (id: NavId) => void;
  planName?: string;
  isGuest?: boolean;
  hasSubscription?: boolean;
  onUpgrade: () => void;
};

export function Sidebar({
  active,
  onNavigate,
  planName,
  isGuest,
  hasSubscription = true,
  onUpgrade,
}: Props) {
  const planLabel = isGuest
    ? "Free trial"
    : hasSubscription
      ? (planName ?? "Active plan")
      : "No plan";

  return (
    <aside className="sidebar">
      <div className="sidebar__brand">
        <div className="sidebar__logo" aria-hidden>
          IP
        </div>
        <div className="sidebar__brand-text">
          <span className="sidebar__title">IPNOVA</span>
          <span className="sidebar__plan">{planLabel}</span>
        </div>
      </div>

      <nav className="sidebar__nav" aria-label="Main">
        {NAV.map((item) => {
          const isActive = active === item.id;
          return (
            <button
              key={item.id}
              type="button"
              className={`sidebar__link ${isActive ? "sidebar__link--active" : ""}`}
              onClick={() => onNavigate(item.id)}
              aria-current={isActive ? "page" : undefined}
            >
              <span className="sidebar__link-icon">{item.icon}</span>
              <span className="sidebar__link-label">{item.label}</span>
            </button>
          );
        })}
      </nav>

      <div className="sidebar__footer">
        <button type="button" className="sidebar-upgrade" onClick={onUpgrade}>
          <span className="sidebar-upgrade__label">Launch offer</span>
          <span className="sidebar-upgrade__price">
            €{LAUNCH_OFFER.priceEur.toFixed(2)}
            <span className="sidebar-upgrade__period"> / 3 mo</span>
          </span>
        </button>
      </div>
    </aside>
  );
}
