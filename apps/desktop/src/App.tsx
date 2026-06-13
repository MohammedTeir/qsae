import React, { useState } from "react";
import { Routes, Route } from "react-router-dom";
import Home from "./pages/Home";
import Progress from "./pages/Progress";
import Results from "./pages/Results";
import { CompressionResult } from "./types";

export interface AppState {
  compressionResult: CompressionResult | null;
  setCompressionResult: (result: CompressionResult | null) => void;
}

export const AppContext = React.createContext<AppState | null>(null);

function App() {
  const [compressionResult, setCompressionResult] = useState<CompressionResult | null>(null);

  return (
    <AppContext.Provider value={{ compressionResult, setCompressionResult }}>
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100">
        <header className="bg-qsae-700 text-white py-4 px-6 shadow-lg">
          <div className="flex items-center justify-between max-w-6xl mx-auto">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 bg-white rounded-lg flex items-center justify-center">
                <span className="text-qsae-700 font-bold text-lg">Q</span>
              </div>
              <div>
                <h1 className="text-xl font-bold">QSAE</h1>
                <p className="text-xs text-qsae-200">Quorum Sensing Adaptive Encoder</p>
              </div>
            </div>
          </div>
        </header>

        <main className="max-w-6xl mx-auto p-6">
          <Routes>
            <Route path="/" element={<Home />} />
            <Route path="/progress" element={<Progress />} />
            <Route path="/results" element={<Results />} />
          </Routes>
        </main>
      </div>
    </AppContext.Provider>
  );
}

export default App;
