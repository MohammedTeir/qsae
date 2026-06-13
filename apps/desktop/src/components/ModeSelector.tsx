import React, { useState } from "react";
import { Zap, Scale, Gauge } from "lucide-react";

interface ModeSelectorProps {
  onChange: (mode: "fast" | "balanced" | "max") => void;
}

const ModeSelector: React.FC<ModeSelectorProps> = ({ onChange }) => {
  const [selected, setSelected] = useState<"fast" | "balanced" | "max">("balanced");

  const modes = [
    {
      id: "fast" as const,
      label: "Fast",
      description: "Larger blocks, fewer switches",
      icon: Zap,
      lambda: 0.8,
      delta: 2.0,
    },
    {
      id: "balanced" as const,
      label: "Balanced",
      description: "Default settings",
      icon: Scale,
      lambda: 0.5,
      delta: 1.2,
    },
    {
      id: "max" as const,
      label: "Max Ratio",
      description: "Smaller blocks, aggressive switching",
      icon: Gauge,
      lambda: 0.3,
      delta: 0.8,
    },
  ];

  const handleSelect = (mode: "fast" | "balanced" | "max") => {
    setSelected(mode);
    onChange(mode);
  };

  return (
    <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
      {modes.map((mode) => {
        const Icon = mode.icon;
        const isSelected = selected === mode.id;

        return (
          <button
            key={mode.id}
            onClick={() => handleSelect(mode.id)}
            className={`p-4 rounded-lg border-2 transition-all text-left ${
              isSelected
                ? "border-qsae-600 bg-qsae-50 dark:bg-qsae-900/20"
                : "border-gray-200 dark:border-gray-700 hover:border-gray-300"
            }`}
          >
            <div className="flex items-center gap-3 mb-2">
              <Icon className={`w-5 h-5 ${isSelected ? "text-qsae-600" : "text-gray-400"}`} />
              <span className={`font-semibold ${isSelected ? "text-qsae-700" : "text-gray-700 dark:text-gray-300"}`}>
                {mode.label}
              </span>
            </div>
            <p className="text-xs text-gray-500 dark:text-gray-400">{mode.description}</p>
            <div className="mt-2 text-xs text-gray-400">
              λ={mode.lambda} | δ={mode.delta}
            </div>
          </button>
        );
      })}
    </div>
  );
};

export default ModeSelector;
