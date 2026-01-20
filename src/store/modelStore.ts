import { create } from 'zustand';

interface ModelStore {
  selectedModel: string | null;
  
  setSelectedModel: (model: string) => void;
  clearSelectedModel: () => void;
}

export const useModelStore = create<ModelStore>((set) => ({
  selectedModel: null,
  
  setSelectedModel: (model: string) => set({ selectedModel: model }),
  clearSelectedModel: () => set({ selectedModel: null }),
}));
