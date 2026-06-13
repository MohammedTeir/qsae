import React from "react";

interface EntropyHeatmapProps {
  data: number[];
}

const EntropyHeatmap: React.FC<EntropyHeatmapProps> = ({ data }) => {
  if (!data || data.length === 0) {
    return (
      <div className="h-48 bg-gray-100 dark:bg-gray-800 rounded-lg flex items-center justify-center">
        <p className="text-gray-400">No entropy data</p>
      </div>
    );
  }

  const getColor = (entropy: number): string => {
    const t = Math.min(entropy / 8, 1);
    // HSL: Green (120°) → Yellow (60°) → Red (0°)
    const hue = (1 - t) * 120;
    return `hsl(${hue}, 80%, 50%)`;
  };

  // Responsive grid sizing
  const maxCols = Math.min(50, Math.ceil(Math.sqrt(data.length * 2)));
  const blockSize = Math.max(8, Math.min(24, Math.floor(600 / maxCols)));

  return (
    <div className="space-y-4">
      {/* Legend */}
      <div className="flex items-center gap-3 text-xs">
        <span className="text-gray-500">Low (0.0)</span>
        <div className="flex-1 h-4 rounded-full overflow-hidden" style={{
          background: "linear-gradient(to right, hsl(120,80%,50%), hsl(60,80%,50%), hsl(0,80%,50%))"
        }} />
        <span className="text-gray-500">High (8.0)</span>
      </div>

      {/* Grid */}
      <div className="flex flex-wrap gap-1" style={{ maxWidth: "100%" }}>
        {data.map((entropy, i) => (
          <div
            key={i}
            className="rounded-sm transition-all hover:scale-125 hover:z-10"
            style={{
              width: blockSize,
              height: blockSize,
              backgroundColor: getColor(entropy),
            }}
            title={`Block ${i}: H = ${entropy.toFixed(2)}`}
          />
        ))}
      </div>

      {/* Stats */}
      <div className="grid grid-cols-3 gap-4 text-sm">
        <div className="bg-green-50 dark:bg-green-900/20 p-3 rounded-lg">
          <p className="text-green-600 font-medium">Min</p>
          <p className="text-2xl font-bold">{Math.min(...data).toFixed(2)}</p>
        </div>
        <div className="bg-yellow-50 dark:bg-yellow-900/20 p-3 rounded-lg">
          <p className="text-yellow-600 font-medium">Avg</p>
          <p className="text-2xl font-bold">{(data.reduce((a, b) => a + b, 0) / data.length).toFixed(2)}</p>
        </div>
        <div className="bg-red-50 dark:bg-red-900/20 p-3 rounded-lg">
          <p className="text-red-600 font-medium">Max</p>
          <p className="text-2xl font-bold">{Math.max(...data).toFixed(2)}</p>
        </div>
      </div>
    </div>
  );
};

export default EntropyHeatmap;
