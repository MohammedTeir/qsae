import React from "react";

interface EntropyHeatmapProps {
  data: number[];
}

const EntropyHeatmap: React.FC<EntropyHeatmapProps> = ({ data }) => {
  if (!data || data.length === 0) {
    return (
      <div className="h-32 bg-gray-100 dark:bg-gray-800 rounded-md flex items-center justify-center">
        <p className="text-sm text-gray-400">No entropy data available</p>
      </div>
    );
  }

  // Normalize entropy values (0-8) to colors
  const getColor = (entropy: number): string => {
    // Green (low) → Yellow (mid) → Red (high)
    const normalized = Math.min(entropy / 8, 1);

    if (normalized < 0.33) {
      // Green to Yellow
      const t = normalized / 0.33;
      return `rgb(${Math.round(255 * t)}, ${Math.round(200 + 55 * (1 - t))}, 0)`;
    } else if (normalized < 0.66) {
      // Yellow to Orange
      const t = (normalized - 0.33) / 0.33;
      return `rgb(255, ${Math.round(200 * (1 - t))}, 0)`;
    } else {
      // Orange to Red
      const t = (normalized - 0.66) / 0.34;
      return `rgb(255, ${Math.round(100 * (1 - t))}, 0)`;
    }
  };

  // Calculate block width based on data length
  const maxBlocks = 100;
  const blockWidth = Math.max(4, Math.min(20, Math.floor(600 / Math.min(data.length, maxBlocks))));
  const displayData = data.length > maxBlocks 
    ? data.filter((_, i) => i % Math.ceil(data.length / maxBlocks) === 0)
    : data;

  return (
    <div className="space-y-2">
      {/* Legend */}
      <div className="flex items-center gap-2 text-xs">
        <span className="text-gray-500">0.0</span>
        <div className="flex-1 h-3 rounded-full overflow-hidden">
          <div 
            className="h-full w-full"
            style={{
              background: "linear-gradient(to right, rgb(0,200,0), rgb(255,200,0), rgb(255,100,0), rgb(255,0,0))"
            }}
          />
        </div>
        <span className="text-gray-500">8.0</span>
      </div>

      {/* Heatmap */}
      <div className="flex flex-wrap gap-0.5">
        {displayData.map((entropy, index) => (
          <div
            key={index}
            className="rounded-sm transition-colors"
            style={{
              width: `${blockWidth}px`,
              height: `${blockWidth}px`,
              backgroundColor: getColor(entropy),
            }}
            title={`Block ${index}: H = ${entropy.toFixed(2)}`}
          />
        ))}
      </div>

      {/* Stats */}
      <div className="flex gap-4 text-xs text-gray-500">
        <span>Min: {Math.min(...data).toFixed(2)}</span>
        <span>Max: {Math.max(...data).toFixed(2)}</span>
        <span>Avg: {(data.reduce((a, b) => a + b, 0) / data.length).toFixed(2)}</span>
      </div>
    </div>
  );
};

export default EntropyHeatmap;
