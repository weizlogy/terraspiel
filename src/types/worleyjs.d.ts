declare module 'worleyjs' {
  function worley(
    point: number[], 
    options?: { 
      seed?: number; 
      maxDistance?: number; 
      distance?: 'euclidean' | 'manhattan' | 'chebyshev';
    }
  ): [number, number];
  
  export default worley;
}