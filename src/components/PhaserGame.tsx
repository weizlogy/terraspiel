import { useEffect, useRef } from 'react';
import Phaser from 'phaser';
import useGameStore from '../stores/gameStore';
import { ELEMENTS, type ElementName, type Particle, type MoveDirection } from '../types/elements';
import { simulatePhysics, calculateStats, simulateParticles } from '../game/physics';
import { varyColor } from '../utils/colors';

const PARTICLE_ELEMENTS: ElementName[] = ['FIRE'];

class GameScene extends Phaser.Scene {
  private grid: ElementName[][] = [];
  private lastMoveGrid: MoveDirection[][] = [];
  private colorGrid: string[][] = [];
  private particles: Particle[] = []; // Add state for particles
  private width: number = 80;
  private height: number = 60;
  private cellSize: number = 8;
  private gridGraphics!: Phaser.GameObjects.Graphics;
  private isDrawing: boolean = false;
  private eraseMode: boolean = false;
  private lastSimulationTime: number = 0;
  private simulationInterval: number = 30; // ms - even faster physics updates for smoother movement
  private lastDrawTime: number = 0;
  private drawInterval: number = 30; // ms - more responsive drawing when holding mouse

  constructor() {
    super('GameScene');
  }

  init() {
    // Get initial state from store
    const state = useGameStore.getState();
    this.grid = state.grid;
    this.lastMoveGrid = state.lastMoveGrid;
    this.colorGrid = state.colorGrid;
    this.particles = state.particles;
    this.width = state.width;
    this.height = state.height;
    
    // Calculate cell size based on window dimensions to fit the grid
    this.adjustCellSize();
  }

  create() {
    // Create graphics object for drawing the grid and particles
    this.gridGraphics = this.add.graphics();
    
    // Set up input handling
    this.input.on('pointerdown', this.handlePointerDown, this);
    this.input.on('pointermove', this.handlePointerMove, this);
    this.input.on('pointerup', this.handlePointerUp, this);
    
    // Subscribe to store updates for grid and particles
    useGameStore.subscribe((state) => {
      this.grid = state.grid;
      this.lastMoveGrid = state.lastMoveGrid;
      this.colorGrid = state.colorGrid;
      this.particles = state.particles;
      this.width = state.width;
      this.height = state.height;
      this.renderAll(); // Re-render whenever state changes
    });
    
    // Listen for window resize to adjust cell size
    window.addEventListener('resize', () => {
      this.adjustCellSize();
    });
    
    // Initial render
    this.renderAll();
  }
  
  private adjustCellSize() {
    // Calculate new cell size based on current window dimensions
    const availableWidth = window.innerWidth;
    const availableHeight = window.innerHeight - 120; // Account for toolbar and footer
    
    const cellWidth = Math.floor(availableWidth / this.width);
    const cellHeight = Math.floor(availableHeight / this.height);
    
    // Use the smaller of the two to ensure the grid fits in the window
    this.cellSize = Math.max(1, Math.min(cellWidth, cellHeight, 10)); // Max cell size of 10, min of 1
  }

  update(time: number) {
    // Update FPS counter
    const fps = Math.round(this.game.loop.actualFps);
    useGameStore.getState().setFps(fps);
    
    // Run physics simulation at intervals
    if (time - this.lastSimulationTime > this.simulationInterval) {
      this.lastSimulationTime = time;
      this.runSimulation();
    }
    
    // Handle continuous drawing when mouse is held down
    if (this.isDrawing) {
      const pointer = this.input.activePointer;
      if (pointer.isDown && time - this.lastDrawTime > this.drawInterval) {
        this.lastDrawTime = time;
        this.updateAtPointer(pointer.x, pointer.y);
      }
    }
  }

  private runSimulation() {
    // Simulate grid and particles
    const { newGrid, newLastMoveGrid, newColorGrid } = simulatePhysics(this.grid, this.lastMoveGrid, this.colorGrid);
    const newParticles = simulateParticles(this.particles, this.grid);

    // Update store with new states
    const state = useGameStore.getState();
    state.setGrid(newGrid);
    state.setLastMoveGrid(newLastMoveGrid);
    state.setColorGrid(newColorGrid);
    state.setParticles(newParticles);
    
    // Update stats from grid
    const stats = calculateStats(newGrid);
    state.updateStats(stats);
  }

  private handlePointerDown(pointer: Phaser.Input.Pointer) {
    this.isDrawing = true;
    this.eraseMode = pointer.rightButtonDown();
    this.updateAtPointer(pointer.x, pointer.y);
  }

  private handlePointerMove(pointer: Phaser.Input.Pointer) {
    if (this.isDrawing) {
      this.updateAtPointer(pointer.x, pointer.y);
    }
  }

  private handlePointerUp() {
    this.isDrawing = false;
  }

  private updateAtPointer(x: number, y: number) {
    const gridX = Math.floor(x / this.cellSize);
    const gridY = Math.floor(y / this.cellSize);
    
    // Check bounds
    if (gridX < 0 || gridX >= this.width || gridY < 0 || gridY >= this.height) {
      return;
    }
    
    const state = useGameStore.getState();
    const newGrid = [...this.grid.map(row => [...row])];
    const newColorGrid = [...this.colorGrid.map(row => [...row])];
    
    if (this.eraseMode) {
      newGrid[gridY][gridX] = 'EMPTY';
      newColorGrid[gridY][gridX] = ELEMENTS.EMPTY.color;
      state.setGrid(newGrid);
      state.setColorGrid(newColorGrid);
    } else {
      const selectedElement = state.selectedElement;
      // If selected element is a particle type, add a particle. Otherwise, update the grid.
      if (PARTICLE_ELEMENTS.includes(selectedElement)) {
        // Add particle with some initial random velocity
        const vx = (Math.random() - 0.5) * 0.5;
        const vy = (Math.random() - 0.5) * 0.5;
        state.addParticle(gridX + 0.5, gridY + 0.5, selectedElement, vx, vy);
      } else {
        newGrid[gridY][gridX] = selectedElement;
        const baseColor = ELEMENTS[selectedElement].color;
        newColorGrid[gridY][gridX] = selectedElement !== 'EMPTY' ? varyColor(baseColor) : baseColor;
        state.setGrid(newGrid);
        state.setColorGrid(newColorGrid);
        
        // Update stats after grid change
        const stats = calculateStats(newGrid);
        state.updateStats(stats);
      }
    }
  }

  private renderAll() {
    this.gridGraphics.clear();
    
    // 1. Render the grid cells
    for (let y = 0; y < this.height; y++) {
      for (let x = 0; x < this.width; x++) {
        const elementName = this.grid[y][x];
        if (elementName !== 'EMPTY') {
          const color = this.colorGrid[y][x];
          this.gridGraphics.fillStyle(parseInt(color.replace('#', '0x')), 1);
          this.gridGraphics.fillRect(
            x * this.cellSize,
            y * this.cellSize,
            this.cellSize,
            this.cellSize
          );
        }
      }
    }
    
    // 2. Render the particles
    for (const particle of this.particles) {
      const element = ELEMENTS[particle.type];
      this.gridGraphics.fillStyle(parseInt(element.color.replace('#', '0x')), 1);
      // Render particle as a smaller circle in the center of its floating position
      this.gridGraphics.fillCircle(
        particle.px * this.cellSize,
        particle.py * this.cellSize,
        this.cellSize * 0.5 // radius
      );
    }
  }
}


const PhaserGame: React.FC = () => {
  const gameContainerRef = useRef<HTMLDivElement>(null);
  const gameRef = useRef<Phaser.Game | null>(null);

  useEffect(() => {
    if (!gameContainerRef.current) return;

    const config: Phaser.Types.Core.GameConfig = {
      type: Phaser.WEBGL,
      width: window.innerWidth,
      height: window.innerHeight - 120, // Account for toolbar and footer
      parent: gameContainerRef.current,
      backgroundColor: '#000000',
      scene: GameScene,
      scale: {
        mode: Phaser.Scale.FIT, // Fit to container but without animation
        autoCenter: Phaser.Scale.CENTER_BOTH,
        parent: gameContainerRef.current
      }
    };

    gameRef.current = new Phaser.Game(config);

    return () => {
      if (gameRef.current) {
        gameRef.current.destroy(true);
        gameRef.current = null;
      }
    };
  }, []);

  return (
    <div className="relative flex-1 w-full">
      <div ref={gameContainerRef} className="game-container w-full h-full" />
      <div className="absolute top-2 right-2 z-10">
        <div className="bg-black bg-opacity-50 text-white p-2 rounded text-sm">
          <div>Left Click: Place Element</div>
          <div>Right Click: Erase</div>
        </div>
      </div>
    </div>
  );
};

export default PhaserGame;