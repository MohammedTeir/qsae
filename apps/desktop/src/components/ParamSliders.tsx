import React from "react";
import { CompressionParams } from "../types";

interface ParamSlidersProps {
  params: CompressionParams;
  onChange: (params: CompressionParams) => void;
}

const ParamSliders: React.FC<ParamSlidersProps> = ({ params, onChange }) => {
  const handleChange = (key: keyof CompressionParams, value: number | boolean) => {
    onChange({ ...params, [key]: value });
  };

  return (
    <div className="space-y-6">
      {/* Lambda Slider */}
      <div>
        <div className="flex justify-between mb-2">
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Adaptation Speed (Lambda)
          </label>
          <span className="text-sm text-qsae-600 font-mono">{params.lambda.toFixed(1)}</span>
        </div>
        <input
          type="range"
          min="0.1"
          max="2.0"
          step="0.1"
          value={params.lambda}
          onChange={(e) => handleChange("lambda", parseFloat(e.target.value))}
          className="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none cursor-pointer accent-qsae-600"
        />
        <p className="text-xs text-gray-500 mt-1">
          Controls how quickly the compression adapts to new data types. High values react instantly to changes, while low values stay steady over larger sections.
        </p>
      </div>

      {/* Delta Slider */}
      <div>
        <div className="flex justify-between mb-2">
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Switch Hesitation (Delta)
          </label>
          <span className="text-sm text-qsae-600 font-mono">{params.delta.toFixed(1)}</span>
        </div>
        <input
          type="range"
          min="0.5"
          max="3.0"
          step="0.1"
          value={params.delta}
          onChange={(e) => handleChange("delta", parseFloat(e.target.value))}
          className="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none cursor-pointer accent-qsae-600"
        />
        <p className="text-xs text-gray-500 mt-1">
          Controls how sure the algorithm must be before changing compression methods. High values avoid frequent switching to save processing time; low values switch immediately for minor gains.
        </p>
      </div>

      {/* Block Size Slider */}
      <div>
        <div className="flex justify-between mb-2">
          <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
            Processing Chunk Size
          </label>
          <span className="text-sm text-qsae-600 font-mono">
            {(params.block_size / 1024).toFixed(0)} KB
          </span>
        </div>
        <input
          type="range"
          min="4096"
          max="262144"
          step="4096"
          value={params.block_size}
          onChange={(e) => handleChange("block_size", parseInt(e.target.value))}
          className="w-full h-2 bg-gray-200 dark:bg-gray-700 rounded-lg appearance-none cursor-pointer accent-qsae-600"
        />
        <p className="text-xs text-gray-500 mt-1">
          The size of each data block processed at a time. Large blocks are faster and use less memory; small blocks adapt better to mixed files but take longer to process.
        </p>
      </div>

      {/* Quorum Toggle */}
      <div className="flex items-center gap-3">
        <input
          type="checkbox"
          id="use-quorum"
          checked={params.use_quorum}
          onChange={(e) => handleChange("use_quorum", e.target.checked)}
          className="w-4 h-4 text-qsae-600 rounded border-gray-300 focus:ring-qsae-500"
        />
        <label htmlFor="use-quorum" className="text-sm font-medium text-gray-700 dark:text-gray-300">
          Use Quorum Sensing
        </label>
      </div>
    </div>
  );
};

export default ParamSliders;
