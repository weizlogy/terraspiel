import { create } from 'zustand';
import { ELEMENTS, type ElementName, type Particle, type MoveDirection } from '../types/elements';
import { varyColor } from '../utils/colors';

interface GameState {
  selectedElement: ElementName;
  isPlaying: boolean;
  grid: ElementName[][];
  lastMoveGrid: MoveDirection[][];
  colorGrid: string[][];
  particles: Particle[];
  nextParticleId: number;
  width: number;
  height: number;
  stats: Record<ElementName, number>;
  fps: number;
  setSelectedElement: (element: ElementName) => void;
  setGrid: (grid: ElementName[][]) => void;
  setLastMoveGrid: (lastMoveGrid: MoveDirection[][]) => void;
  setColorGrid: (colorGrid: string[][]) => void;
  setParticles: (particles: Particle[]) => void;
  addParticle: (x: number, y: number, type: ElementName, vx?: number, vy?: number) => void;
  initializeGrid: (width: number, height: number) => void;
  clearGrid: () => void;
  randomizeGrid: () => void;
  updateStats: (stats: Record<ElementName, number>) => void;
  setFps: (fps: number) => void;
}

const useGameStore = create<GameState>()((set, get) => ({
  selectedElement: 'SOIL',
  isPlaying: true,
  grid: [],
  lastMoveGrid: [],
  colorGrid: [],
  particles: [],
  nextParticleId: 0,
  width: 80,
  height: 60,
  stats: {
    EMPTY: 0, // Will be set to 0 to not count EMPTY elements
    SOIL: 0,
    WATER: 0,
    FIRE: 0,
    WET_SOIL: 0,
    STEAM: 0,
  },
  fps: 0,
  setSelectedElement: (element) => set({ selectedElement: element }),
  setGrid: (grid) => set({ grid }),
  setLastMoveGrid: (lastMoveGrid) => set({ lastMoveGrid }),
  setColorGrid: (colorGrid) => set({ colorGrid }),
  setParticles: (particles) => set({ particles }),
  addParticle: (x, y, type, vx = 0, vy = 0) => {
    const newParticle: Particle = {
      id: get().nextParticleId,
      px: x,
      py: y,
      vx,
      vy,
      type,
      life: 1000, // Default lifespan
    };
    set((state) => ({
      particles: [...state.particles, newParticle],
      nextParticleId: state.nextParticleId + 1,
    }));
  },
  initializeGrid: (width, height) => {
    const grid: ElementName[][] = Array(height)
      .fill(null)
      .map(() => Array(width).fill('EMPTY'));
    const lastMoveGrid: MoveDirection[][] = Array(height)
      .fill(null)
      .map(() => Array(width).fill('NONE'));
    const colorGrid: string[][] = Array(height)
      .fill(null)
      .map(() => Array(width).fill(ELEMENTS.EMPTY.color));
    
    // Initialize stats (don't count EMPTY)
    const stats: Record<ElementName, number> = {
      EMPTY: 0,
      SOIL: 0,
      WATER: 0,
      FIRE: 0,
      WET_SOIL: 0,
      STEAM: 0,
    };
    
    set({ grid, lastMoveGrid, colorGrid, width, height, stats, particles: [], nextParticleId: 0 });
  },
  clearGrid: () => set((state) => {
    const grid: ElementName[][] = Array(state.height)
      .fill(null)
      .map(() => Array(state.width).fill('EMPTY'));
    const lastMoveGrid: MoveDirection[][] = Array(state.height)
      .fill(null)
      .map(() => Array(state.width).fill('NONE'));
    const colorGrid: string[][] = Array(state.height)
      .fill(null)
      .map(() => Array(state.width).fill(ELEMENTS.EMPTY.color));
      
    const stats: Record<ElementName, number> = {
      EMPTY: 0, // Don't count EMPTY
      SOIL: 0,
      WATER: 0,
      FIRE: 0,
      WET_SOIL: 0,
      STEAM: 0,
    };
    
    return { grid, lastMoveGrid, colorGrid, stats, particles: [], nextParticleId: 0 };
  }),
  randomizeGrid: () => set((state) => {
    const elements: ElementName[] = ['EMPTY', 'SOIL', 'WATER', 'FIRE'];
    const grid: ElementName[][] = [];
    const lastMoveGrid: MoveDirection[][] = [];
    const colorGrid: string[][] = [];
    const stats: Record<ElementName, number> = {
      EMPTY: 0, // Don't count EMPTY
      SOIL: 0,
      WATER: 0,
      FIRE: 0,
      WET_SOIL: 0,
      STEAM: 0,
    };
    
    for (let y = 0; y < state.height; y++) {
      const row: ElementName[] = [];
      const moveRow: MoveDirection[] = [];
      const colorRow: string[] = [];
      for (let x = 0; x < state.width; x++) {
        const randomElement = elements[Math.floor(Math.random() * elements.length)];
        row.push(randomElement);
        moveRow.push('NONE');
        const baseColor = ELEMENTS[randomElement].color;
        colorRow.push(randomElement !== 'EMPTY' ? varyColor(baseColor) : baseColor);
        if (randomElement !== 'EMPTY') {
          stats[randomElement]++;
        }
      }
      grid.push(row);
      lastMoveGrid.push(moveRow);
      colorGrid.push(colorRow);
    }
    
    return { grid, lastMoveGrid, colorGrid, stats, particles: [], nextParticleId: 0 };
  }),
  updateStats: (stats) => set({ stats }),
  setFps: (fps) => set({ fps })
}));

export default useGameStore;