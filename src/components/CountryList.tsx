import { motion } from "framer-motion";

import { COUNTRIES } from "../data/countries";
import type { Country } from "../data/countries";

type Props = {
  selectedId: string;
  onSelect: (country: Country) => void;
};

export function CountryList({ selectedId, onSelect }: Props) {
  return (
    <div className="country-list">
      <p className="country-list__title">Servers (display)</p>
      <p className="country-hint country-hint--list">
        Changing country here does not switch your Marzban node yet.
      </p>
      <ul>
        {COUNTRIES.map((country, i) => {
          const selected = country.id === selectedId;
          return (
            <motion.li
              key={country.id}
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: i * 0.05 }}
            >
              <button
                type="button"
                className={`country-item ${selected ? "country-item--on" : ""} ${!country.available ? "country-item--disabled" : ""}`}
                onClick={() => onSelect(country)}
                disabled={!country.available}
              >
                <span className="country-item__flag">{country.flag}</span>
                <span className="country-item__info">
                  <span className="country-item__name">{country.name}</span>
                  <span className="country-item__meta">
                    {country.city}
                    {country.ping != null ? ` · ${country.ping} ms` : ""}
                  </span>
                </span>
                {!country.available ? (
                  <span className="country-item__soon">Soon</span>
                ) : null}
              </button>
            </motion.li>
          );
        })}
      </ul>
    </div>
  );
}
