import { create } from 'zustand';
import { type Element, type ElementName, type Particle, type MoveDirection, type Cell, type ParticleType, type TransformationRule } from '../types/elements';
import { type ParticleInteractionRule } from '../game/behaviors/etherBehavior';
import { varyColor } from '../utils/colors';

import { generateTerrain } from '../utils/worldGenerator';

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
  colorVariations: Map<string, string[]>;
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
  initializeColorVariations: () => void;
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
  colorVariations: new Map(),
  transformationRules: [],
  particleInteractionRules: [],
  perf: { simulationTime: 0, renderTime: 0 },
  setPerf: (perfData: Partial<{ simulationTime: number; renderTime: number; }>) => set((state) => ({ perf: { ...state.perf, ...perfData } })),
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
  randomizeGrid: () => {
    const { width, height, elements, colorVariations } = get();
    const newGrid = generateTerrain({ width, height, seed: Math.random() });
    const newLastMoveGrid: MoveDirection[][] = Array(height).fill(0).map(() => Array(width).fill('NONE'));
    const newColorGrid: string[][] = Array(height).fill(0).map(() => Array(width).fill(elements.EMPTY.color));

    const stats: Record<string, number> = {};
    Object.keys(elements).forEach(name => { stats[name] = 0; });
    stats.ETHER = 0;
    stats.THUNDER = 0;

    // Calculate stats from the generated grid
    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const elementType = newGrid[y][x].type;
        if (elementType !== 'EMPTY') {
          stats[elementType] = (stats[elementType] || 0) + 1;
        }
      }
    }

    // Color the grid based on the final elements
    for (let y = 0; y < height; y++) {
      for (let x = 0; x < width; x++) {
        const elementType = newGrid[y][x].type;
        if (elementType !== 'EMPTY') {
          const variations = colorVariations.get(elementType);
          if (variations && variations.length > 0) {
            newColorGrid[y][x] = variations[Math.floor(Math.random() * variations.length)];
          } else {
            newColorGrid[y][x] = elements[elementType].color;
          }
        }
      }
    }

    set({ grid: newGrid, lastMoveGrid: newLastMoveGrid, colorGrid: newColorGrid, stats: stats as any, particles: [], nextParticleId: 0, updateSource: 'ui' });
  },
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

      get().initializeColorVariations();

      // Now that elements are loaded, initialize the grid
      get().initializeGrid();
      // get().randomizeGrid(); // Removed this line

    } catch (error) {
      console.error("Error loading elements:", error);
    }
  },
  initializeColorVariations: () => {
    const { elements } = get();
    const newColorVariations = new Map<string, string[]>();
    const variationCount = 10;

    for (const elementName in elements) {
      const element = elements[elementName as ElementName];
      
      if (element.hasColorVariation && element.color) {
        const variations: string[] = [];
        for (let i = 0; i < variationCount; i++) {
          variations.push(varyColor(element.color));
        }
        newColorVariations.set(element.name, variations);
      }

      if (element.partColors) {
        for (const partName in element.partColors) {
          const partColor = element.partColors[partName];
          const variations: string[] = [];
          for (let i = 0; i < variationCount; i++) {
            variations.push(varyColor(partColor));
          }
          newColorVariations.set(`${element.name}_${partName}`, variations);
        }
      }
    }
    set({ colorVariations: newColorVariations as any });
    console.log('Color variations initialized.', newColorVariations);
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