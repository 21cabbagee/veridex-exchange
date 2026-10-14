// Real-time Order Book & Trade Depth Component
class OrderBookView {
  constructor(containerId) {
    this.asksContainer = document.getElementById('book-asks');
    this.bidsContainer = document.getElementById('book-bids');
    this.spreadContainer = document.getElementById('book-spread');
    this.tradesContainer = document.getElementById('trades-list');
  }

  updateBook(data) {
    if (!data) return;
    const asks = (data.asks || []).slice().reverse(); // Show lowest ask at the bottom
    const bids = data.bids || [];

    // Max depth for percentage bars
    const maxQty = Math.max(
      ...asks.map(a => parseFloat(a.quantity || 0)),
      ...bids.map(b => parseFloat(b.quantity || 0)),
      1
    );

    if (this.asksContainer) {
      this.asksContainer.innerHTML = asks.map(a => {
        const p = parseFloat(a.price);
        const q = parseFloat(a.quantity);
        const pct = Math.min(100, (q / maxQty) * 100);
        return `
          <div class="book-row ask" onclick="window.app?.setPrice('${p.toFixed(2)}')">
            <div class="depth-bar" style="width: ${pct}%"></div>
            <span>${(p * 100).toFixed(0)}¢</span>
            <span>${q.toLocaleString()}</span>
            <span>$${(p * q).toFixed(0)}</span>
          </div>
        `;
      }).join('');
    }

    if (this.bidsContainer) {
      this.bidsContainer.innerHTML = bids.map(b => {
        const p = parseFloat(b.price);
        const q = parseFloat(b.quantity);
        const pct = Math.min(100, (q / maxQty) * 100);
        return `
          <div class="book-row bid" onclick="window.app?.setPrice('${p.toFixed(2)}')">
            <div class="depth-bar" style="width: ${pct}%"></div>
            <span>${(p * 100).toFixed(0)}¢</span>
            <span>${q.toLocaleString()}</span>
            <span>$${(p * q).toFixed(0)}</span>
          </div>
        `;
      }).join('');
    }

    if (this.spreadContainer && asks.length > 0 && bids.length > 0) {
      const bestAsk = parseFloat(asks[asks.length - 1].price);
      const bestBid = parseFloat(bids[0].price);
      const spread = (bestAsk - bestBid) * 100;
      this.spreadContainer.innerText = `SPREAD: ${spread.toFixed(1)}¢ (${(bestBid * 100).toFixed(0)}¢ / ${(bestAsk * 100).toFixed(0)}¢)`;
    }
  }

  updateTrades(trades) {
    if (!this.tradesContainer || !trades) return;
    this.tradesContainer.innerHTML = trades.slice(0, 15).map(t => {
      const p = parseFloat(t.price);
      const q = parseFloat(t.quantity);
      const isBuy = t.side === 'BUY';
      const time = new Date(t.ts || Date.now()).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
      return `
        <div class="trade-row ${isBuy ? 'buy' : 'sell'}">
          <span>${(p * 100).toFixed(0)}¢</span>
          <span>${q.toLocaleString()} sh</span>
          <span class="trade-time">${time}</span>
        </div>
      `;
    }).join('');
  }
}

window.OrderBookView = OrderBookView;
