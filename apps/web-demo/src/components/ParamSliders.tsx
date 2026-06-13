import React from "react";

interface ParamSlidersProps {
  params: { lambda: number; delta: number; block_size: number };
  onChange: (params: { lambda: number; delta: number; block_size: number }) => void;
}

const ParamSliders: React.FC<ParamSlidersProps> = ({ params, onChange }) => {
  const handleChange = (key: keyof typeof params, value: number) => {
    onChange({ ...params, [key]: value });
  };

  return (
    <div className="space-y-6">
      <div>
        <div className="flex justify-between mb-2">
          <label className="text-sm font-medium">Lambda (λ) — Decay</label>
          <span className="text-sm text-qsae-600 font-mono">{params.lambda.toFixed(1)}</span>
        </div>
        <input
          type="range"
          min="0.1"
          max="2.0"
          step="0.1"
          value={params.lambda}
          onChange={(e) => handleChange("lambda", parseFloat(e.target.value))}
          className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-qsae-600"
        />
        <p className="text-xs text-gray-500 mt-1">Higher = more local sensitivity</p>
      </div>

      <div>
        <div className="flex justify-between mb-2">
          <label className="text-sm font-medium">Delta (δ) — Threshold</label>
          <span className="text-sm text-qsae-600 font-mono">{params.delta.toFixed(1)}</span>
        </div>
        <input
          type="range"
          min="0.5"
          max="3.0"
          step="0.1"
          value={params.delta}
          onChange={(e) => handleChange("delta", parseFloat(e.target.value))}
          className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-qsae-600"
        />
        <p className="text-xs text-gray-500 mt-1">Higher = fewer switches</p>
      </div>

      <div>
        <div className="flex justify-between mb-2">
          <label className="text-sm font-medium">Block Size</label>
          <span className="text-sm text-qsae-600 font-mono">{(params.block_size / 1024).toFixed(0)} KB</span>
        </div>
        <input
          type="range"
          min="4096"
          max="262144"
          step="4096"
          value={params.block_size}
          onChange={(e) => handleChange("block_size", parseInt(e.target.value))}
          className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-qsae-600"
        />
        <p className="text-xs text-gray-500 mt-1">Larger = faster, less granular</p>
      </div>
    </div>
  );
};

export default ParamSliders;
