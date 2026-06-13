import { useState, useEffect } from "react";

export interface WasmModule {
  loaded: boolean;
  version: string;
  compressAndAnalyze: (data: Uint8Array, lambda: number, delta: number, blockSize: number) => Promise<any>;
  analyzeOnly: (data: Uint8Array, lambda: number, delta: number, blockSize: number) => Promise<any>;
  decompress: (data: Uint8Array) => Promise<Uint8Array>;
  inspect: (data: Uint8Array) => Promise<any>;
}

export const useWasm = () => {
  const [module, setModule] = useState<WasmModule | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const loadWasm = async () => {
      try {
        // Dynamic import of wasm-pack output
        // After running `npm run wasm`, the module is at ./wasm/qsae_wasm.js
        const wasmModule = await import("../wasm/qsae_wasm");
        await wasmModule.default();

        setModule({
          loaded: true,
          version: wasmModule.version(),
          compressAndAnalyze: async (data, lambda, delta, blockSize) => {
            return wasmModule.compress_and_analyze(data, lambda, delta, blockSize);
          },
          analyzeOnly: async (data, lambda, delta, blockSize) => {
            return wasmModule.analyze_only(data, lambda, delta, blockSize);
          },
          decompress: async (data) => {
            return wasmModule.decompress(data);
          },
          inspect: async (data) => {
            return wasmModule.inspect(data);
          },
        });
      } catch (err) {
        setError(err instanceof Error ? err.message : "Failed to load WASM module");
      } finally {
        setLoading(false);
      }
    };

    loadWasm();
  }, []);

  return { module, loading, error };
};
