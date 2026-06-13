import React from "react";
import { WasmAnalysisResult } from "../types";

interface StatsPanelProps {
  result: WasmAnalysisResult;
}

const StatsPanel: React.FC<StatsPanelProps> = ({ result }) => {
  const formatBytes = (bytes: number): string => {
    if (bytes === 0) return "0 B";
    const k = 1024;
    const sizes = ["B", "KB", "MB", "GB"];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return `${(bytes / Math.pow(k, i)).toFixed(2)} ${sizes[i]}`;
  };

  const savings = ((1 - 1 / result.ratio) * 100).toFixed(1);
  const speed = result.duration_ms > 0
    ? ((result.original_size / 1024 / 1024) / (result.duration_ms / 1000)).toFixed(1)
    : "0";

  return (
    <div className="space-y-6">
      {/* Main Stats */}
      <div className="grid grid-cols-2 gap-4">
        <div className="bg-qsae-50 dark:bg-qsae-900/20 p-4 rounded-lg">
          <p className="text-sm text-qsae-600 font-medium">Compression Ratio</p>
          <p className="text-3xl font-bold text-qsae-700">{result.ratio.toFixed(2)}:1</p>
          <p className="text-sm text-green-600">{savings}% smaller</p>
        </div>
        <div className="bg-gray-50 dark:bg-gray-800 p-4 rounded-lg">
          <p className="text-sm text-gray-600 font-medium">Processing Speed</p>
          <p className="text-3xl font-bold">{speed}</p>
          <p className="text-sm text-gray-500">MB/s</p>
        </div>
      </div>

      {/* Detailed Metrics */}
      <div className="space-y-3">
        <h4 className="font-semibold text-gray-700 dark:text-gray-300">Detailed Metrics</h4>

        <div className="grid grid-cols-2 gap-3 text-sm">
          <div className="flex justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded">
            <span className="text-gray-500">Original Size</span>
            <span className="font-medium">{formatBytes(result.original_size)}</span>
          </div>
          <div className="flex justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded">
            <span className="text-gray-500">Compressed</span>
            <span className="font-medium">{formatBytes(result.compressed_size)}</span>
          </div>
          <div className="flex justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded">
            <span className="text-gray-500">Blocks</span>
            <span className="font-medium">{result.block_count}</span>
          </div>
          <div className="flex justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded">
            <span className="text-gray-500">Duration</span>
            <span className="font-medium">{(result.duration_ms / 1000).toFixed(2)}s</span>
          </div>
          <div className="flex justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded">
            <span className="text-gray-500">Switch Points</span>
            <span className="font-medium">{result.switch_points.length}</span>
          </div>
          <div className="flex justify-between p-3 bg-gray-50 dark:bg-gray-800 rounded">
            <span className="text-gray-500">Codecs Used</span>
            <span className="font-medium">{result.codec_names.length}</span>
          </div>
        </div>
      </div>

      {/* Codec Table */}
      <div>
        <h4 className="font-semibold text-gray-700 dark:text-gray-300 mb-3">Codec Distribution</h4>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-200 dark:border-gray-700">
                <th className="text-left py-2 px-3">Codec</th>
                <th className="text-right py-2 px-3">Blocks</th>
                <th className="text-right py-2 px-3">Percentage</th>
              </tr>
            </thead>
            <tbody>
              {result.codec_names.map((name, i) => (
                <tr key={name} className="border-b border-gray-100 dark:border-gray-800">
                  <td className="py-2 px-3 font-medium">{name}</td>
                  <td className="text-right py-2 px-3">{result.codec_counts[i]}</td>
                  <td className="text-right py-2 px-3">
                    {((result.codec_counts[i] / result.block_count) * 100).toFixed(1)}%
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
};

export default StatsPanel;
