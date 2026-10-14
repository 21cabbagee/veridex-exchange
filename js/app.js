// VERIDEX High-Velocity Exchange Frontend Application
class VeridexApp {
  constructor() {
    this.currentView = 'markets';
    this.selectedMarket = null;
    this.currentOutcome = 'YES';
    this.currentSide = 'BUY';
    this.chart = null;
    this.orderbook = null;
    this.markets = [];
    this.positions = JSON.parse(localStorage.getItem('veridex_positions') || '[]');
  }

  async init() {
    this.bindEvents();
    this.orderbook = new OrderBookView();
    await this.loadMarkets();
    this.initChart();
    this.renderPositions();
    this.renderLeaderboard();

    // Connect WebSocket
    window.veridex.connectWs((event) => this.handleWsMessage(event));
  }

  bindEvents() {
    // Nav Tabs
    document.querySelectorAll('.nav-tab').forEach(tab => {
      tab.addEventListener('click', (e) => {
        const view = e.currentTarget.getAttribute('data-view');
        this.switchView(view);
      });
    });

    // Category pills
    document.querySelectorAll('.pill').forEach(pill => {
      pill.addEventListener('click', (e) => {
        document.querySelectorAll('.pill').forEach(p => p.classList.remove('active'));
        e.currentTarget.classList.add('active');
        const cat = e.currentTarget.getAttribute('data-cat');
        this.filterMarkets(cat);
      });
    });

    // Search input
    document.getElementById('search-input')?.addEventListener('input', (e) => {
      this.searchMarkets(e.target.value);
    });

    // Hotkey '/' to focus search
    window.addEventListener('keydown', (e) => {
      if (e.key === '/' && document.activeElement?.tagName !== 'INPUT') {
        e.preventDefault();
        document.getElementById('search-input')?.focus();
      }
    });

    // Back button
    document.getElementById('back-btn')?.addEventListener('click', () => {
      this.switchView('markets');
    });

    // Ticket outcome & side selectors
    document.getElementById('ttab-yes')?.addEventListener('click', () => this.setOutcome('YES'));
    document.getElementById('ttab-no')?.addEventListener('click', () => this.setOutcome('NO'));
    document.getElementById('side-buy')?.addEventListener('click', () => this.setSide('BUY'));
    document.getElementById('side-sell')?.addEventListener('click', () => this.setSide('SELL'));

    document.getElementById('price-inc')?.addEventListener('click', () => this.adjustPrice(0.01));
    document.getElementById('price-dec')?.addEventListener('click', () => this.adjustPrice(-0.01));
    document.getElementById('price-input')?.addEventListener('input', () => this.updateCostSummary());
    document.getElementById('qty-input')?.addEventListener('input', () => this.updateCostSummary());

    document.getElementById('btn-trade')?.addEventListener('click', () => this.submitOrder());

    // Auth modal
    document.getElementById('btn-login')?.addEventListener('click', () => this.openAuthModal('login'));
    document.getElementById('btn-signup')?.addEventListener('click', () => this.openAuthModal('register'));
    document.getElementById('modal-close')?.addEventListener('click', () => this.closeAuthModal());
    document.getElementById('btn-demo')?.addEventListener('click', () => this.loginDemo());
  }

  switchView(view) {
    this.currentView = view;
    document.querySelectorAll('.view').forEach(v => v.classList.remove('active'));
    document.querySelectorAll('.nav-tab').forEach(t => t.classList.remove('active'));

    const targetView = document.getElementById(`view-${view}`);
    const targetTab = document.getElementById(`tab-${view}`);
    if (targetView) targetView.classList.add('active');
    if (targetTab) targetTab.classList.add('active');

    window.scrollTo({ top: 0, behavior: 'smooth' });
  }

  async loadMarkets() {
    const res = await window.veridex.getMarkets();
    this.markets = res.markets || [];
    this.renderMarkets(this.markets);
    this.updateStatsBar();
  }

  updateStatsBar() {
    let totalVol = 0;
    let totalOI = 0;
    let totalTraders = 0;

    this.markets.forEach(m => {
      totalVol += parseFloat(m.volume_24h || 0);
      totalOI += parseFloat(m.liquidity || 0);
      totalTraders += parseInt(m.num_traders || 0);
    });

    document.getElementById('stat-volume').innerText = `$${(totalVol / 1000000).toFixed(1)}M`;
    document.getElementById('stat-oi').innerText = `$${(totalOI / 1000000).toFixed(1)}M`;
    document.getElementById('stat-markets').innerText = this.markets.length;
    document.getElementById('stat-traders').innerText = totalTraders.toLocaleString();
  }

  filterMarkets(category) {
    if (!category) {
      this.renderMarkets(this.markets);
    } else {
      const filtered = this.markets.filter(m => (m.category || '').toLowerCase() === category.toLowerCase());
      this.renderMarkets(filtered);
    }
  }

  searchMarkets(query) {
    const q = query.toLowerCase().trim();
    if (!q) {
      this.renderMarkets(this.markets);
      return;
    }
    const filtered = this.markets.filter(m => 
      m.title.toLowerCase().includes(q) || (m.category || '').toLowerCase().includes(q)
    );
    this.renderMarkets(filtered);
  }

  renderMarkets(list) {
    const grid = document.getElementById('market-grid');
    if (!grid) return;

    if (list.length === 0) {
      grid.innerHTML = `<div style="grid-column: 1/-1; text-align: center; padding: 60px; color: var(--text-muted)">No active contracts found for this category.</div>`;
      return;
    }

    grid.innerHTML = list.map(m => {
      const yesP = parseFloat(m.yes_price || 0.5);
      const noP = parseFloat(m.no_price || 0.5);
      const yesPct = (yesP * 100).toFixed(0);
      const noPct = (noP * 100).toFixed(0);
      const volM = (parseFloat(m.volume_24h || 0) / 1000000).toFixed(1);

      return `
        <div class="market-card" onclick="window.app.selectMarket('${m.id}')">
          <div class="card-img-placeholder">
            <span>${this.getCategoryIcon(m.category)}</span>
          </div>
          <div class="card-body">
            <div class="card-cat">${m.category || 'General'}</div>
            <div class="card-title">${m.title}</div>
            
            <div class="card-prices">
              <div class="price-box price-box-yes">
                <div class="price-outcome">YES</div>
                <div class="price-val">${yesPct}¢</div>
                <div class="price-sub">${yesPct}% probability</div>
              </div>
              <div class="price-box price-box-no">
                <div class="price-outcome">NO</div>
                <div class="price-val">${noPct}¢</div>
                <div class="price-sub">${noPct}% probability</div>
              </div>
            </div>

            <div class="card-prob-bar">
              <div class="card-prob-fill" style="width: ${yesPct}%"></div>
            </div>

            <div class="card-stats">
              <div class="card-stat">
                <div class="card-stat-val">$${volM}M</div>
                <div class="card-stat-label">24h Vol</div>
              </div>
              <div class="card-stat">
                <div class="card-stat-val">${(m.num_traders || 1200).toLocaleString()}</div>
                <div class="card-stat-label">Traders</div>
              </div>
              <div class="card-stat">
                <div class="card-stat-val">$${(parseFloat(m.liquidity || 500000) / 1000).toFixed(0)}k</div>
                <div class="card-stat-label">Liquidity</div>
              </div>
            </div>
          </div>
          <div class="card-footer">
            <span class="card-traders">Synthetic Binary / Zero-Arb</span>
            <span class="card-badge badge-active">${m.status || 'Active'}</span>
          </div>
        </div>
      `;
    }).join('');
  }

  getCategoryIcon(cat) {
    switch ((cat || '').toLowerCase()) {
      case 'politics': return '🗳';
      case 'crypto': return '₿';
      case 'finance': return '📈';
      case 'sports': return '⚽';
      case 'science': return '🔬';
      default: return '◈';
    }
  }

  async selectMarket(id) {
    const market = this.markets.find(m => m.id === id) || (await window.veridex.getMarket(id))?.market;
    if (!market) return;
    this.selectedMarket = market;

    document.getElementById('detail-cat').innerText = (market.category || 'General').toUpperCase();
    document.getElementById('detail-title').innerText = market.title;
    document.getElementById('detail-desc').innerText = market.description || 'Binary event derivative settled to $1.00.';
    document.getElementById('detail-vol').innerText = `24h Vol: $${(parseFloat(market.volume_24h || 0) / 1000000).toFixed(2)}M`;
    document.getElementById('detail-traders').innerText = `Traders: ${(market.num_traders || 0).toLocaleString()}`;
    document.getElementById('detail-liquidity').innerText = `Liquidity: $${(parseFloat(market.liquidity || 0) / 1000).toFixed(0)}k`;
    document.getElementById('detail-oracle').innerText = `Oracle: ${market.resolver || 'VERIDEX Oracle'}`;

    const yesPrice = parseFloat(market.yes_price || 0.73);
    document.getElementById('price-input').value = yesPrice.toFixed(2);
    document.getElementById('chart-current-price').innerText = `${(yesPrice * 100).toFixed(0)}% (${(yesPrice * 100).toFixed(0)}¢)`;

    this.switchView('detail');

    // Load orderbook and trades
    const bookData = await window.veridex.getOrderbook(id);
    const tradesData = await window.veridex.getTrades(id);
    this.orderbook.updateBook(bookData.yes);
    this.orderbook.updateTrades(tradesData.trades);

    // Populate chart history
    this.generateChartHistory(yesPrice);
    this.updateCostSummary();
  }

  initChart() {
    this.chart = new PriceChart('price-chart');
  }

  generateChartHistory(basePrice) {
    if (!this.chart) return;
    const points = [];
    let p = basePrice - 0.12;
    for (let i = 0; i < 35; i++) {
      p += (Math.random() - 0.48) * 0.015;
      p = Math.max(0.05, Math.min(0.95, p));
      points.push({ time: Date.now() - (35 - i) * 60000, price: p });
    }
    points.push({ time: Date.now(), price: basePrice });
    this.chart.setData(points);
  }

  setOutcome(outcome) {
    this.currentOutcome = outcome;
    document.getElementById('ttab-yes')?.classList.toggle('active', outcome === 'YES');
    document.getElementById('ttab-no')?.classList.toggle('active', outcome === 'NO');

    const btn = document.getElementById('btn-trade');
    if (btn) btn.innerText = `${this.currentSide === 'BUY' ? 'Buy' : 'Sell'} ${outcome} @ ${(parseFloat(document.getElementById('price-input').value) * 100).toFixed(0)}¢`;

    if (this.selectedMarket) {
      const p = outcome === 'YES' ? parseFloat(this.selectedMarket.yes_price) : parseFloat(this.selectedMarket.no_price);
      document.getElementById('price-input').value = p.toFixed(2);
    }
    this.updateCostSummary();
  }

  setSide(side) {
    this.currentSide = side;
    document.getElementById('side-buy')?.classList.toggle('active', side === 'BUY');
    document.getElementById('side-sell')?.classList.toggle('active', side === 'SELL');

    const btn = document.getElementById('btn-trade');
    if (btn) {
      btn.classList.toggle('sell', side === 'SELL');
      btn.innerText = `${side === 'BUY' ? 'Buy' : 'Sell'} ${this.currentOutcome} @ ${(parseFloat(document.getElementById('price-input').value) * 100).toFixed(0)}¢`;
    }
    this.updateCostSummary();
  }

  setPrice(price) {
    document.getElementById('price-input').value = price;
    this.updateCostSummary();
  }

  adjustPrice(delta) {
    const input = document.getElementById('price-input');
    let val = parseFloat(input.value || 0.5) + delta;
    val = Math.max(0.01, Math.min(0.99, val));
    input.value = val.toFixed(2);
    this.updateCostSummary();
  }

  updateCostSummary() {
    const price = parseFloat(document.getElementById('price-input')?.value || 0.5);
    const qty = parseInt(document.getElementById('qty-input')?.value || 100);

    const cost = price * qty;
    const payout = qty * 1.0;
    const maxReturn = cost > 0 ? ((payout - cost) / cost * 100) : 0;

    document.getElementById('cost-value').innerText = `$${cost.toFixed(2)}`;
    document.getElementById('payout-value').innerText = `$${payout.toFixed(2)}`;
    document.getElementById('return-value').innerText = `+${maxReturn.toFixed(1)}%`;

    const probPct = (price * 100).toFixed(0);
    const probBar = document.getElementById('prob-bar-fill');
    if (probBar) probBar.style.width = `${probPct}%`;
    const probLabel = document.getElementById('prob-label');
    if (probLabel) probLabel.innerText = `${probPct}¢ (${probPct}% Implied)`;

    const btn = document.getElementById('btn-trade');
    if (btn) {
      btn.innerText = `${this.currentSide === 'BUY' ? 'Buy' : 'Sell'} ${this.currentOutcome} @ ${probPct}¢`;
    }
  }

  async submitOrder() {
    if (!this.selectedMarket) return;
    const price = parseFloat(document.getElementById('price-input').value);
    const qty = parseInt(document.getElementById('qty-input').value);
    const orderType = document.getElementById('order-type-select').value;

    const res = await window.veridex.placeOrder({
      market_id: this.selectedMarket.id,
      outcome: this.currentOutcome,
      side: this.currentSide,
      order_type: orderType,
      price: price,
      quantity: qty.toString(),
    });

    this.showToast(`Order matched on VERIDEX engine! Filled ${qty} ${this.currentOutcome} @ ${(price * 100).toFixed(0)}¢`, 'success');

    // Add to local positions
    this.positions.push({
      marketId: this.selectedMarket.id,
      title: this.selectedMarket.title,
      outcome: this.currentOutcome,
      shares: qty,
      avgPrice: price,
      currentPrice: price,
    });
    localStorage.setItem('veridex_positions', JSON.stringify(this.positions));
    this.renderPositions();

    if (this.chart) this.chart.addPoint(price);
  }

  renderPositions() {
    const tbody = document.getElementById('positions-tbody');
    if (!tbody) return;

    if (this.positions.length === 0) {
      tbody.innerHTML = `<tr class="empty-row"><td colspan="7">No open positions. Submit orders in the Markets tab!</td></tr>`;
      return;
    }

    let totalVal = 0;
    tbody.innerHTML = this.positions.map(p => {
      const curVal = p.shares * p.currentPrice;
      const cost = p.shares * p.avgPrice;
      const pnl = curVal - cost;
      totalVal += curVal;

      return `
        <tr>
          <td style="font-family: var(--font-sans); font-weight: 700;">${p.title}</td>
          <td><span class="card-badge ${p.outcome === 'YES' ? 'badge-active' : ''}">${p.outcome}</span></td>
          <td>${p.shares}</td>
          <td>${(p.avgPrice * 100).toFixed(0)}¢</td>
          <td>${(p.currentPrice * 100).toFixed(0)}¢</td>
          <td class="${pnl >= 0 ? 'pnl-positive' : 'pnl-negative'}">${pnl >= 0 ? '+' : ''}$${pnl.toFixed(2)}</td>
          <td><button class="btn-ghost" style="padding: 4px 10px; font-size: 11px;">Liquidate</button></td>
        </tr>
      `;
    }).join('');

    document.getElementById('port-total').innerText = `$${(10000 + totalVal).toFixed(2)}`;
  }

  renderLeaderboard() {
    const lb = document.getElementById('lb-grid');
    if (!lb) return;

    const leaders = [
      { rank: 1, name: 'citadel_alpha.eth', volume: '$24.8M', pnl: '+$5.42M', winrate: '89.2%' },
      { rank: 2, name: 'veridex_maker_01', volume: '$19.3M', pnl: '+$3.81M', winrate: '84.0%' },
      { rank: 3, name: 'macro_arb_fund', volume: '$14.6M', pnl: '+$2.75M', winrate: '78.5%' },
      { rank: 4, name: 'chicago_probability', volume: '$10.2M', pnl: '+$1.94M', winrate: '71.2%' },
      { rank: 5, name: 'quantum_veritas', volume: '$8.5M', pnl: '+$1.48M', winrate: '68.9%' },
    ];

    lb.innerHTML = leaders.map(l => `
      <div class="lb-row">
        <div class="lb-rank ${l.rank === 1 ? 'gold' : l.rank === 2 ? 'silver' : l.rank === 3 ? 'bronze' : ''}">#${l.rank}</div>
        <div class="lb-name">${l.name}</div>
        <div class="lb-val">${l.volume}</div>
        <div class="lb-val pnl-positive">${l.pnl}</div>
        <div class="lb-val">${l.winrate}</div>
      </div>
    `).join('');
  }

  handleWsMessage(ev) {
    if (ev.type === 'PRICE_TICK' && this.selectedMarket && ev.market_id === this.selectedMarket.id) {
      if (this.chart) this.chart.addPoint(parseFloat(ev.price));
    }
  }

  openAuthModal(mode) {
    const modal = document.getElementById('auth-modal');
    modal?.classList.remove('hidden');
    document.getElementById('modal-title').innerText = mode === 'register' ? 'Register Account' : 'Authenticate';
    document.getElementById('reg-username-field')?.classList.toggle('hidden', mode !== 'register');
  }

  closeAuthModal() {
    document.getElementById('auth-modal')?.classList.add('hidden');
  }

  loginDemo() {
    window.veridex.setAuth('demo_token', { id: 'demo_user', username: 'VerifiedTrader' });
    this.closeAuthModal();
    this.showToast('Authenticated into VERIDEX Demo with $10,000 collateral!', 'success');
  }

  showToast(msg, type = 'info') {
    const container = document.getElementById('toast-container');
    if (!container) return;
    const toast = document.createElement('div');
    toast.className = `toast ${type}`;
    toast.innerHTML = `<span class="toast-icon">⟐</span><span class="toast-msg">${msg}</span>`;
    container.appendChild(toast);
    setTimeout(() => {
      toast.classList.add('toast-fade');
      setTimeout(() => toast.remove(), 300);
    }, 4500);
  }
}

window.addEventListener('DOMContentLoaded', () => {
  window.app = new VeridexApp();
  window.app.init();
});
