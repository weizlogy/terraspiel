import { create } from 'zustand';
import { type Element, type ElementName, type Particle, type MoveDirection, type Cell, type ParticleType, type TransformationRule } from '../types/elements';
import { type ParticleInteractionRule } from '../game/behaviors/etherBehavior';
import { varyColor } from '../utils/colors';
import { createNoise2D } from '../utils/noise';

interface GameState {
  selectedElement: ElementName;
  isPlaying: boolean;
  grid: Cell[][];
  lastMoveGrid: MoveDirection[][];
  colorGrid: string[][];
  particles: Particle[];
  nextParticleId: number;
  width: number;
  height: number;
  stats: Record<ElementName | 'ETHER' | 'THUNDER', number>;
  fps: number;
  elements: Record<ElementName, Element>;
  transformationRules: TransformationRule[];
  particleInteractionRules: ParticleInteractionRule[];
  perf: { simulationTime: number; renderTime: number; };
  updateSource?: 'ui' | 'simulation'; // Add this line
  setSelectedElement: (element: ElementName) => void;
  setGrid: (grid: Cell[][]) => void;
  setLastMoveGrid: (lastMoveGrid: MoveDirection[][]) => void;
  setColorGrid: (colorGrid: string[][]) => void;
  setParticles: (particles: Particle[]) => void;
  setSimulationResult: (data: {
    newGrid: Cell[][];
    newLastMoveGrid: MoveDirection[][];
    newColorGrid: string[][];
    newParticles: Particle[];
  }) => void;
  addParticle: (x: number, y: number, type: ParticleType, vx?: number, vy?: number) => void;
  initializeGrid: () => void;
  clearGrid: () => void;
  randomizeGrid: () => void;
  updateStats: (stats: Record<ElementName | 'ETHER' | 'THUNDER', number>) => void;
  setFps: (fps: number) => void;
  loadElements: () => Promise<void>;
  loadRules: () => Promise<void>;
}

const FIXED_WIDTH = 320;
const FIXED_HEIGHT = 180;

const useGameStore = create<GameState>()((set, get) => ({
  selectedElement: 'SOIL',
  isPlaying: true,
  grid: [],
  lastMoveGrid: [],
  colorGrid: [],
  particles: [],
  nextParticleId: 0,
  width: FIXED_WIDTH,
  height: FIXED_HEIGHT,
  stats: {
    EMPTY: 0, // Will be set to 0 to not count EMPTY elements
    SOIL: 0,
    WATER: 0,
    MUD: 0,
    FERTILE_SOIL: 0,
    PEAT: 0,
    CLOUD: 0,
    CLAY: 0,
    FIRE: 0,
    SAND: 0,
    STONE: 0,
    SEED: 0,
    PLANT: 0,
    OIL: 0,
    MAGMA: 0,
    CRYSTAL: 0,
    BASALT: 0,
    OBSIDIAN: 0,
    ELECTRUM: 0,
    RUBY: 0,
    SAPPHIRE: 0,
    AMETHYST: 0,
    GARNET: 0,
    EMERALD: 0,
    ETHER: 0,
    THUNDER: 0,
  },
  fps: 0,
  elements: {} as Record<ElementName, Element>,
  transformationRules: [],
  particleInteractionRules: [],
  perf: { simulationTime: 0, renderTime: 0 },
  setPerf: (perfData) => set((state) => ({ perf: { ...state.perf, ...perfData } })),
  setSelectedElement: (element) => set({ selectedElement: element }),
  setGrid: (grid) => set({ grid, updateSource: 'ui' }),
  setLastMoveGrid: (lastMoveGrid) => set({ lastMoveGrid }),
  setColorGrid: (colorGrid) => set({ colorGrid }),
  setParticles: (particles) => set({ particles }),
  setSimulationResult: (data) => set({
    grid: data.newGrid,
    lastMoveGrid: data.newLastMoveGrid,
    colorGrid: data.newColorGrid,
    particles: data.newParticles,
    updateSource: 'simulation',
  }),
  addParticle: (x, y, type, vx = 0, vy = 0) => {
    let life = 150; // Default for ETHER
    if (type === 'THUNDER') {
      life = 20; // Specific lifespan for THUNDER
    }

    const newParticle: Particle = {
      id: get().nextParticleId,
      px: x,
      py: y,
      vx,
      vy,
      type,
      life: life,
    };
    set((state) => ({
      particles: [...state.particles, newParticle],
      nextParticleId: state.nextParticleId + 1,
    }));
  },
  initializeGrid: () => {
    const width = FIXED_WIDTH;
    const height = FIXED_HEIGHT;
    const grid: Cell[][] = Array(height)
      .fill(0) // Fill with a primitive value to ensure distinct array references
      .map(() => Array(width).fill(0).map(() => ({ type: 'EMPTY' })));
    const lastMoveGrid: MoveDirection[][] = Array(height)
      .fill(0)
      .map(() => Array(width).fill('NONE'));
    const colorGrid: string[][] = Array(height)
      .fill(0)
      .map(() => Array(width).fill(get().elements.EMPTY.color));

    // Initialize stats dynamically from loaded elements
    const elements = get().elements;
    const stats: Record<string, number> = {};
    Object.keys(elements).forEach(name => {
      stats[name] = 0;
    });
    stats.ETHER = 0; // Add ETHER separately as it's a particle
    stats.THUNDER = 0; // Add THUNDER separately as it's a particle

    set({ grid, lastMoveGrid, colorGrid, width, height, stats: stats as Record<ElementName | 'ETHER' | 'THUNDER', number>, particles: [], nextParticleId: 0, updateSource: 'ui' });
  },
  clearGrid: () => set((state) => {
    const grid: Cell[][] = Array(state.height)
      .fill(0)
      .map(() => Array(state.width).fill(0).map(() => ({ type: 'EMPTY' })));
    const lastMoveGrid: MoveDirection[][] = Array(state.height)
      .fill(0)
      .map(() => Array(state.width).fill('NONE'));
    const colorGrid: string[][] = Array(state.height)
      .fill(0)
      .map(() => Array(state.width).fill(get().elements.EMPTY.color));

    const elements = get().elements;
    const stats: Record<string, number> = {};
    Object.keys(elements).forEach(name => {
      stats[name] = 0;
    });
    stats.ETHER = 0; // Add ETHER separately as it's a particle
    stats.THUNDER = 0; // Add THUNDER separately as it's a particle

    return { grid, lastMoveGrid, colorGrid, stats: stats as Record<ElementName | 'ETHER' | 'THUNDER', number>, particles: [], nextParticleId: 0, updateSource: 'ui' };
  }),
  randomizeGrid: () => set((state) => {
    const { width, height, elements } = state;
    const newGrid: Cell[][] = Array(height).fill(0).map(() => Array(width).fill(0).map(() => ({ type: 'EMPTY' })));
    const newLastMoveGrid: MoveDirection[][] = Array(height).fill(0).map(() => Array(width).fill('NONE'));
    const newColorGrid: string[][] = Array(height).fill(0).map(() => Array(width).fill(elements.EMPTY.color));

    const stats: Record<string, number> = {};
    Object.keys(elements).forEach(name => { stats[name] = 0; });
    stats.ETHER = 0;
    stats.THUNDER = 0;

    const noise2D = createNoise2D(() => Math.random());
    const featureFrequency = 80;
    const materialFrequency = 25;
    const patchFrequency = 15;
    const veinFrequency = 8; // For very rare veins

    const baseCaveThreshold = 0.5;
    const caveThresholdVariation = 0.1;
    const caveThreshold = baseCaveThreshold + (Math.random() - 0.5) * 2 * caveThresholdVariation;

    const stoneDepth = height * 0.35;

    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const featureNoise = (noise2D(x / featureFrequency, y / featureFrequency) + 1) / 2;
        let elementType: ElementName = 'EMPTY';

        if (featureNoise > caveThreshold) {
          const materialNoise = (noise2D(x / materialFrequency, y / materialFrequency) + 1) / 2;
          const patchNoise = (noise2D(x / patchFrequency, y / patchFrequency) + 1) / 2;

          if (y > stoneDepth + materialNoise * 20) {
            elementType = 'STONE';
          } else {
            elementType = 'SOIL';
          }

          if (elementType === 'SOIL') {
            if (materialNoise > 0.7) elementType = 'CLAY';
            if (patchNoise > 0.85) elementType = 'SAND';
          } else if (elementType === 'STONE') {
            if (materialNoise < 0.3) elementType = 'BASALT';
          }

          if (y > height * 0.6 && patchNoise < 0.15) {
            elementType = 'PEAT';
          }

          // Add rare veins in stone layers
          if (elementType === 'STONE' || elementType === 'BASALT') {
            const veinNoise = (noise2D(x / veinFrequency, y / veinFrequency) + 1) / 2;
            if (veinNoise > 0.95) {
              elementType = 'CRYSTAL';
            } else if (veinNoise > 0.9) {
              elementType = 'OBSIDIAN';
            }
          }

          // Add fertile soil patches near the surface
          if (elementType === 'SOIL' && y < stoneDepth * 1.2) {
            if (patchNoise < 0.1) {
              elementType = 'FERTILE_SOIL';
            }
          }
        }

        if (elementType !== 'EMPTY') {
          newGrid[y][x] = { type: elementType };
          stats[elementType]++;
        }
      }
    }

    // Generate liquid pools at the bottom
    const magmaLevel = Math.floor(height * 0.95);
    const oilLevel = Math.floor(height * 0.9);
    const waterLevel = Math.floor(height * 0.8);

    for (let y = waterLevel; y < height; y++) {
      for (let x = 0; x < width; x++) {
        if (newGrid[y][x].type === 'EMPTY') {
          let liquidType: ElementName | null = null;
          if (y >= magmaLevel && Math.random() < 0.6) {
            liquidType = 'MAGMA';
          } else if (y >= oilLevel && Math.random() < 0.5) {
            liquidType = 'OIL';
          } else {
            liquidType = 'WATER';
          }
          newGrid[y][x] = { type: liquidType };
          stats[liquidType]++;
        }
      }
    }
    
    // Color the grid based on the final elements
    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const elementType = newGrid[y][x].type;
        if (elementType !== 'EMPTY') {
          newColorGrid[y][x] = varyColor(elements[elementType].color);
        }
      }
    }

    return { grid: newGrid, lastMoveGrid: newLastMoveGrid, colorGrid: newColorGrid, stats: stats as any, particles: [], nextParticleId: 0, updateSource: 'ui' };
  }),
  updateStats: (stats) => set({ stats }),
  setFps: (fps) => set({ fps }),
  loadElements: async () => {
    try {
      const response = await fetch('/elements.json');
      if (!response.ok) {
        throw new Error(`Failed to fetch elements: ${response.statusText}`);
      }
      const elementsArray = await response.json();
      const elementsMap = elementsArray.reduce((acc: Record<ElementName, Element>, el: Element) => {
        acc[el.name] = el;
        return acc;
      }, {});
      set({ elements: elementsMap });
      console.log('Elements loaded successfully.', elementsMap);

      // Now that elements are loaded, initialize the grid
      get().initializeGrid();
      // get().randomizeGrid(); // Removed this line

    } catch (error) {
      console.error("Error loading elements:", error);
    }
  },
  loadRules: async () => {
    try {
      const response = await fetch('/rules.json');
      if (!response.ok) {
        throw new Error(`Failed to fetch rules: ${response.statusText}`);
      }
      const rules = await response.json();
      
      const transformationRules = rules.filter((r: any) => r.type !== 'particle_interaction');
      const particleInteractionRules = rules.filter((r: any) => r.type === 'particle_interaction');

      set({ 
        transformationRules: transformationRules,
        particleInteractionRules: particleInteractionRules 
      });
      console.log('Transformation rules loaded:', transformationRules);
      console.log('Particle interaction rules loaded:', particleInteractionRules);
    } catch (error) {
      console.error("Error loading rules:", error);
    }
  },
}));

export default useGameStore;