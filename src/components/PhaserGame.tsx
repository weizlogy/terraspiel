import { useEffect, useRef } from 'react';
import Phaser from 'phaser';
import useGameStore from '../stores/gameStore';
import { ELEMENTS, type ElementName, type Particle, type MoveDirection, type Cell } from '../types/elements';
import { simulateWorld, calculateStats } from '../game/physics';
import { varyColor, blendColors } from '../utils/colors';

const PARTICLE_ELEMENTS: ElementName[] = [];

export class GameScene extends Phaser.Scene {
  private grid: Cell[][] = [];
  private lastMoveGrid: MoveDirection[][] = [];
  private colorGrid: string[][] = [];
  private particles: Particle[] = []; // Add state for particles
  private width: number = 160; // Fixed grid width
  private height: number = 90; // Fixed grid height
  private cellSize: number = 4; // Fixed cell size
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
    
    // Initial render
    this.renderAll();
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
    // Simulate grid and particles together
    const { newGrid, newLastMoveGrid, newColorGrid, newParticles } = simulateWorld(
      this.grid,
      this.lastMoveGrid,
      this.colorGrid,
      this.particles
    );

    // Update store with new states
    const state = useGameStore.getState();
    state.setGrid(newGrid);
    state.setLastMoveGrid(newLastMoveGrid);
    state.setColorGrid(newColorGrid);
    state.setParticles(newParticles);
    
    // Update stats from grid and particles
    const stats = calculateStats(newGrid, newParticles);
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
    const newGrid = this.grid.map(row => row.map(cell => ({ ...cell })));
    const newColorGrid = [...this.colorGrid.map(row => [...row])];
    
    if (this.eraseMode) {
      newGrid[gridY][gridX] = { type: 'EMPTY' };
      newColorGrid[gridY][gridX] = ELEMENTS.EMPTY.color;
      state.setGrid(newGrid);
      state.setColorGrid(newColorGrid);
    } else {
      const selectedElement = state.selectedElement as ElementName;
      // If selected element is a particle type, add a particle. Otherwise, update the grid.p
      if (PARTICLE_ELEMENTS.includes(selectedElement)) {
        // Add particle with some initial random velocity
        const vx = (Math.random() - 0.5) * 0.5;
        const vy = (Math.random() - 0.5) * 0.5;
        state.addParticle(gridX + 0.5, gridY + 0.5, selectedElement, vx, vy);
      } else {
        newGrid[gridY][gridX] = { type: selectedElement };
        const baseColor = ELEMENTS[selectedElement].color;
        newColorGrid[gridY][gridX] = selectedElement !== 'EMPTY' ? varyColor(baseColor) : baseColor;
        state.setGrid(newGrid);
        state.setColorGrid(newColorGrid);
        
        // Update stats after grid change
        const stats = calculateStats(newGrid, this.particles);
        state.updateStats(stats);
      }
    }
  }

  private renderAll() {
    this.gridGraphics.clear();
    
    // 1. Render the grid cells
    for (let y = 0; y < this.height; y++) {
      for (let x = 0; x < this.width; x++) {
        const elementName = this.grid[y][x].type;
        if (elementName !== 'EMPTY') {
          let displayColor = this.colorGrid[y][x];

          // Special rendering for WATER: blend with neighbors
          if (elementName === 'WATER') {
            let blendedColor = displayColor;
            let blendCount = 1; // Start with self color

            // Check 8 neighbors
            for (let dy = -1; dy <= 1; dy++) {
              for (let dx = -1; dx <= 1; dx++) {
                if (dx === 0 && dy === 0) continue;

                const nx = x + dx;
                const ny = y + dy;

                if (nx >= 0 && nx < this.width && ny >= 0 && ny < this.height) {
                  const neighborElement = this.grid[ny][nx].type;
                  if (neighborElement !== 'EMPTY' && neighborElement !== 'WATER') { // Blend with non-empty, non-water neighbors
                    const neighborColor = this.colorGrid[ny][nx];
                    blendedColor = blendColors(blendedColor, neighborColor, 0.9); // Small weight for neighbor
                    blendCount++;
                  }
                }
              }
            }
            displayColor = blendedColor;
          }

          this.gridGraphics.fillStyle(parseInt(displayColor.replace('#', '0x')), 1);
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
      const particleType = particle.type;
      if (particleType === 'EMPTY') {
        continue;
      }

      if (particleType === 'ETHER') {
        const baseAlpha = Math.max(0, particle.life / 150); // Fade out as it dies
        const baseRadius = this.cellSize * (0.2 + (particle.life / 150) * 0.8);
        const color = 0xFFFFFF; // White

        // Draw multiple circles to create a soft, glowing effect
        this.gridGraphics.fillStyle(color, baseAlpha * 0.1);
        this.gridGraphics.fillCircle(particle.px * this.cellSize, particle.py * this.cellSize, baseRadius);

        this.gridGraphics.fillStyle(color, baseAlpha * 0.2);
        this.gridGraphics.fillCircle(particle.px * this.cellSize, particle.py * this.cellSize, baseRadius * 0.7);

        this.gridGraphics.fillStyle(color, baseAlpha * 0.5);
        this.gridGraphics.fillCircle(particle.px * this.cellSize, particle.py * this.cellSize, baseRadius * 0.4);

      } else {
        const element = ELEMENTS[particleType as ElementName];
        if (element) {
          const color = parseInt(element.color.replace('#', '0x'));
          const radius = this.cellSize * 0.5;
          this.gridGraphics.fillStyle(color, 1.0);
          this.gridGraphics.fillCircle(
            particle.px * this.cellSize,
            particle.py * this.cellSize,
            radius
          );
        }
      }
    }
  }
}


const PhaserGame: React.FC = () => {
  const gameContainerRef = useRef<HTMLDivElement>(null);
  const gameRef = useRef<Phaser.Game | null>(null);

  useEffect(() => {
    if (!gameContainerRef.current) return;

    // Load transformation rules before starting the game
    useGameStore.getState().loadTransformationRules();

    const config: Phaser.Types.Core.GameConfig = {
      type: Phaser.WEBGL,
      width: 1280,
      height: 720,
      parent: gameContainerRef.current,
      backgroundColor: '#000000',
      scene: GameScene,
      scale: {
        mode: Phaser.Scale.FIT,
        autoCenter: Phaser.Scale.CENTER_HORIZONTALLY,
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
    </div>
  );
};

export default PhaserGame;