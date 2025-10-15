import { useEffect, useRef } from 'react';
import Phaser from 'phaser';
import { GameScene } from '../game/GameScene';

const PhaserGame: React.FC = () => {
  const gameContainerRef = useRef<HTMLDivElement>(null);
  const gameRef = useRef<Phaser.Game | null>(null);

  useEffect(() => {
    if (!gameContainerRef.current) return;

    const config: Phaser.Types.Core.GameConfig = {
      type: Phaser.WEBGL,
      width: 1280,
      height: 720,
      parent: gameContainerRef.current,
      backgroundColor: '#000000',
      scene: GameScene,
      scale: {
        // mode: Phaser.Scale.FIT,
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