// Canvas-based real-time probability line chart
class PriceChart {
  constructor(canvasId) {
    this.canvas = document.getElementById(canvasId);
    if (!canvasId || !this.canvas) return;
    this.ctx = this.canvas.getContext('2d');
    this.dataPoints = [];
    this.resize();
    window.addEventListener('resize', () => this.resize());
  }

  resize() {
    if (!this.canvas) return;
    const rect = this.canvas.parentElement.getBoundingClientRect();
    this.width = rect.width - 40;
    this.height = 260;
    this.canvas.width = this.width * window.devicePixelRatio;
    this.canvas.height = this.height * window.devicePixelRatio;
    this.ctx.scale(window.devicePixelRatio, window.devicePixelRatio);
    this.render();
  }

  setData(points) {
    this.dataPoints = points;
    this.render();
  }

  addPoint(val) {
    this.dataPoints.push({ time: Date.now(), price: val });
    if (this.dataPoints.length > 50) this.dataPoints.shift();
    this.render();
  }

  render() {
    if (!this.ctx || this.dataPoints.length === 0) return;
    const ctx = this.ctx;
    ctx.clearRect(0, 0, this.width, this.height);

    const prices = this.dataPoints.map(p => p.price);
    const minP = Math.max(0, Math.min(...prices) - 0.05);
    const maxP = Math.min(1.0, Math.max(...prices) + 0.05);
    const range = (maxP - minP) || 0.1;

    // Draw horizontal grid lines
    ctx.strokeStyle = 'rgba(255, 255, 255, 0.05)';
    ctx.lineWidth = 1;
    for (let i = 0; i <= 4; i++) {
      const y = (this.height / 4) * i;
      ctx.beginPath();
      ctx.moveTo(0, y);
      ctx.lineTo(this.width, y);
      ctx.stroke();

      const labelVal = (maxP - (range / 4) * i) * 100;
      ctx.fillStyle = '#4e5468';
      ctx.font = '10px Inter, sans-serif';
      ctx.fillText(`${labelVal.toFixed(0)}%`, 8, y + 12);
    }

    // Coordinates mapping
    const stepX = this.width / (this.dataPoints.length - 1 || 1);
    const points = this.dataPoints.map((p, i) => ({
      x: i * stepX,
      y: this.height - ((p.price - minP) / range) * this.height,
    }));

    // Area gradient
    const grad = ctx.createLinearGradient(0, 0, 0, this.height);
    grad.addColorStop(0, 'rgba(0, 210, 122, 0.25)');
    grad.addColorStop(1, 'rgba(0, 210, 122, 0.0)');

    ctx.beginPath();
    ctx.moveTo(points[0].x, points[0].y);
    for (let i = 1; i < points.length; i++) {
      const prev = points[i - 1];
      const cur = points[i];
      const mx = (prev.x + cur.x) / 2;
      ctx.quadraticCurveTo(prev.x, prev.y, mx, (prev.y + cur.y) / 2);
    }
    ctx.lineTo(points[points.length - 1].x, points[points.length - 1].y);
    ctx.lineTo(this.width, this.height);
    ctx.lineTo(0, this.height);
    ctx.closePath();
    ctx.fillStyle = grad;
    ctx.fill();

    // Line stroke
    ctx.beginPath();
    ctx.moveTo(points[0].x, points[0].y);
    for (let i = 1; i < points.length; i++) {
      const prev = points[i - 1];
      const cur = points[i];
      const mx = (prev.x + cur.x) / 2;
      ctx.quadraticCurveTo(prev.x, prev.y, mx, (prev.y + cur.y) / 2);
    }
    ctx.lineTo(points[points.length - 1].x, points[points.length - 1].y);
    ctx.strokeStyle = '#00d27a';
    ctx.lineWidth = 2.5;
    ctx.stroke();

    // Highlight current price dot
    const last = points[points.length - 1];
    ctx.beginPath();
    ctx.arc(last.x, last.y, 4, 0, Math.PI * 2);
    ctx.fillStyle = '#00d27a';
    ctx.fill();
    ctx.strokeStyle = '#0a0b0f';
    ctx.lineWidth = 2;
    ctx.stroke();
  }
}

window.PriceChart = PriceChart;
