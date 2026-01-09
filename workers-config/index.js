/**
 * Williw Workers 入口点（最简版）
 */

export default {
  async fetch(request) {
    return new Response(JSON.stringify({
      status: "healthy",
      message: "Williw Worker is running!",
      timestamp: new Date().toISOString(),
      version: "1.0.0"
    }), {
      headers: { 'Content-Type': 'application/json' }
    });
  }
};
