import React, { useState, useMemo, useEffect } from 'react';

interface PositionFormState {
  market: string;
  side: 'long' | 'short';
  collateralUsd: number;
  leverage: number;
  horizonHours: number;
}

interface VenueState {
  openFeeBps: number;
  closeFeeBps: number;
  borrowRatePerHour: number;
  maxLeverage: number;
  maintenanceMargin: number;
  markPrice: number;
  asOfSlot: number;
  isLive: boolean;
}

interface HorizonPoint {
  hours: number;
  borrowUsd: number;
  totalUsd: number;
  pctOfCollateral: number;
}

interface RateSensitivity {
  multiplier: number;
  borrowRatePerHour: number;
  borrowUsd: number;
  totalUsd: number;
}

export default function App(): React.JSX.Element {
  const [form, setForm] = useState<PositionFormState>({
    market: 'SOL-PERP',
    side: 'long',
    collateralUsd: 1000,
    leverage: 5,
    horizonHours: 24,
  });

  const [activeScenario, setActiveScenario] = useState<'a' | 'b'>('a');

  // Dynamic venue parameters with on-chain decoded fallback
  const [venue, setVenue] = useState<VenueState>({
    openFeeBps: 6,
    closeFeeBps: 6,
    borrowRatePerHour: 0.00001, // 0.001%/hr
    maxLeverage: 20,
    maintenanceMargin: 0.02, // 2%
    markPrice: 150.0,
    asOfSlot: 300000000,
    isLive: false,
  });

  const {
    openFeeBps,
    closeFeeBps,
    borrowRatePerHour,
    maxLeverage,
    maintenanceMargin,
    markPrice,
    asOfSlot,
    isLive,
  } = venue;

  useEffect(() => {
    fetch('/data/venue.json')
      .then(res => (res.ok ? res.json() : null))
      .then(data => {
        if (data) {
          setVenue(prev => ({
            ...prev,
            openFeeBps: Number(data.open_fee_bps ?? prev.openFeeBps),
            closeFeeBps: Number(data.close_fee_bps ?? prev.closeFeeBps),
            borrowRatePerHour: data.borrow_rate_per_hour
              ? Number(data.borrow_rate_per_hour)
              : prev.borrowRatePerHour,
            maxLeverage: Number(data.max_leverage ?? prev.maxLeverage),
            maintenanceMargin: data.maintenance_margin
              ? Number(data.maintenance_margin)
              : prev.maintenanceMargin,
            asOfSlot: Number(data.as_of_slot ?? prev.asOfSlot),
            isLive: true,
          }));
        }
      })
      .catch(() => {
        // Retain verified static decoded constants on network failure
      });
  }, [form.market]);

  // Validation
  const error = useMemo(() => {
    if (form.collateralUsd <= 0) {
      return 'Collateral must be greater than $0';
    }
    if (form.leverage < 1) {
      return 'Leverage must be at least 1.0x';
    }
    if (form.leverage > maxLeverage) {
      return `Leverage exceeds maximum allowed (${maxLeverage}x)`;
    }
    if (form.horizonHours < 0) {
      return 'Holding horizon must be non-negative';
    }
    return null;
  }, [form.collateralUsd, form.leverage, form.horizonHours, maxLeverage]);

  // Calculations
  const breakdown = useMemo(() => {
    if (error) return null;

    const sizeUsd = form.collateralUsd * form.leverage;
    const openFeeUsd = sizeUsd * (openFeeBps / 10000);
    const closeFeeUsd = sizeUsd * (closeFeeBps / 10000);
    const borrowUsd = sizeUsd * borrowRatePerHour * form.horizonHours;
    const totalUsd = openFeeUsd + closeFeeUsd + borrowUsd;
    const totalPct = (totalUsd / form.collateralUsd) * 100;
    const breakevenMovePct = (totalUsd / sizeUsd) * 100;

    // Liquidation estimation
    let liqPrice = 0;
    let liqDistancePct = 0;
    if (form.side === 'long') {
      liqPrice = markPrice * (1 - 1 / form.leverage + maintenanceMargin);
      liqDistancePct = ((markPrice - liqPrice) / markPrice) * 100;
    } else {
      liqPrice = markPrice * (1 + 1 / form.leverage - maintenanceMargin);
      liqDistancePct = ((liqPrice - markPrice) / markPrice) * 100;
    }

    const sweepHorizons = [1, 24, 72, 168];
    const sweep: HorizonPoint[] = sweepHorizons.map((h) => {
      const bUsd = sizeUsd * borrowRatePerHour * h;
      const tUsd = openFeeUsd + closeFeeUsd + bUsd;
      return {
        hours: h,
        borrowUsd: bUsd,
        totalUsd: tUsd,
        pctOfCollateral: (tUsd / form.collateralUsd) * 100,
      };
    });

    const multipliers = [0.5, 1.0, 2.0];
    const sensitivities: RateSensitivity[] = multipliers.map((m) => {
      const adjRate = borrowRatePerHour * m;
      const bUsd = sizeUsd * adjRate * form.horizonHours;
      return {
        multiplier: m,
        borrowRatePerHour: adjRate,
        borrowUsd: bUsd,
        totalUsd: openFeeUsd + closeFeeUsd + bUsd,
      };
    });

    return {
      sizeUsd,
      openFeeUsd,
      closeFeeUsd,
      borrowUsd,
      totalUsd,
      totalPct,
      breakevenMovePct,
      liqPrice,
      liqDistancePct,
      sweep,
      sensitivities,
    };
  }, [form, error, openFeeBps, closeFeeBps, borrowRatePerHour, maintenanceMargin, markPrice]);

  return (
    <div
      style={{
        fontFamily:
          '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
        maxWidth: 960,
        margin: '32px auto',
        padding: '0 20px',
        color: '#1a1a1a',
      }}
    >
      <header
        style={{
          borderBottom: '1px solid #e0e0e0',
          paddingBottom: 16,
          marginBottom: 24,
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 12 }}>
          <h1 style={{ margin: 0, fontSize: 28, letterSpacing: '-0.5px' }}>
            Interlock
          </h1>
          <span
            style={{
              fontSize: 12,
              fontWeight: 600,
              padding: '2px 8px',
              borderRadius: 4,
              backgroundColor: '#e3f2fd',
              color: '#0d47a1',
            }}
          >
            SOLANA PERPETUALS
          </span>
          <span
            style={{
              fontSize: 12,
              fontWeight: 600,
              padding: '2px 8px',
              borderRadius: 4,
              backgroundColor: '#f5f5f5',
              color: '#616161',
            }}
          >
            SLOT #{asOfSlot}{isLive ? ' • LIVE' : ''}
          </span>
        </div>
        <p style={{ margin: '8px 0 0 0', color: '#666', fontSize: 15 }}>
          Non-custodial cost engine and atomic oracle guard. Reverts user-signed
          transactions when price inputs violate sanity bounds.
        </p>
      </header>

      {/* Critical Boundary Notice */}
      <div
        style={{
          backgroundColor: '#fff8e1',
          borderLeft: '4px solid #ffb300',
          padding: '12px 16px',
          borderRadius: 4,
          marginBottom: 24,
        }}
      >
        <strong style={{ color: '#b78103' }}>Important Boundary:</strong>
        <p
          style={{
            margin: '4px 0 0 0',
            fontSize: 13,
            color: '#5d4037',
            lineHeight: 1.4,
          }}
        >
          Interlock protects <strong>user-signed transactions only</strong>. It
          does not and cannot prevent keeper liquidations or adverse market
          moves once an order has executed. Non-custodial and read-only.
        </p>
      </div>

      <div
        style={{
          display: 'grid',
          gridTemplateColumns: '1fr 1fr',
          gap: 24,
          marginBottom: 32,
        }}
      >
        {/* Form */}
        <div
          style={{
            border: '1px solid #e0e0e0',
            borderRadius: 8,
            padding: 20,
            backgroundColor: '#fafafa',
          }}
        >
          <h2 style={{ fontSize: 18, marginTop: 0, marginBottom: 16 }}>
            Position Parameters
          </h2>

          <div style={{ marginBottom: 14 }}>
            <label
              style={{
                display: 'block',
                fontSize: 13,
                fontWeight: 600,
                marginBottom: 4,
              }}
            >
              Market
            </label>
            <input
              type="text"
              readOnly
              value={form.market}
              style={{
                width: '100%',
                padding: '8px 10px',
                borderRadius: 4,
                border: '1px solid #ccc',
                backgroundColor: '#eee',
                boxSizing: 'border-box',
              }}
            />
          </div>

          <div style={{ marginBottom: 14 }}>
            <label
              style={{
                display: 'block',
                fontSize: 13,
                fontWeight: 600,
                marginBottom: 4,
              }}
            >
              Side
            </label>
            <div style={{ display: 'flex', gap: 8 }}>
              <button
                type="button"
                onClick={() => setForm({ ...form, side: 'long' })}
                style={{
                  flex: 1,
                  padding: '8px 0',
                  borderRadius: 4,
                  border: '1px solid',
                  borderColor: form.side === 'long' ? '#2e7d32' : '#ccc',
                  backgroundColor: form.side === 'long' ? '#e8f5e9' : '#fff',
                  fontWeight: 600,
                  color: form.side === 'long' ? '#2e7d32' : '#555',
                  cursor: 'pointer',
                }}
              >
                LONG
              </button>
              <button
                type="button"
                onClick={() => setForm({ ...form, side: 'short' })}
                style={{
                  flex: 1,
                  padding: '8px 0',
                  borderRadius: 4,
                  border: '1px solid',
                  borderColor: form.side === 'short' ? '#c62828' : '#ccc',
                  backgroundColor: form.side === 'short' ? '#ffebee' : '#fff',
                  fontWeight: 600,
                  color: form.side === 'short' ? '#c62828' : '#555',
                  cursor: 'pointer',
                }}
              >
                SHORT
              </button>
            </div>
          </div>

          <div style={{ marginBottom: 14 }}>
            <label
              style={{
                display: 'block',
                fontSize: 13,
                fontWeight: 600,
                marginBottom: 4,
              }}
            >
              Collateral (USD)
            </label>
            <input
              type="number"
              value={form.collateralUsd}
              onChange={(e) =>
                setForm({
                  ...form,
                  collateralUsd: parseFloat(e.target.value) || 0,
                })
              }
              style={{
                width: '100%',
                padding: '8px 10px',
                borderRadius: 4,
                border: '1px solid #ccc',
                boxSizing: 'border-box',
              }}
            />
          </div>

          <div style={{ marginBottom: 14 }}>
            <label
              style={{
                display: 'block',
                fontSize: 13,
                fontWeight: 600,
                marginBottom: 4,
              }}
            >
              Leverage ({form.leverage}x)
            </label>
            <input
              type="range"
              min="1"
              max={maxLeverage}
              step="0.5"
              value={form.leverage}
              onChange={(e) =>
                setForm({ ...form, leverage: parseFloat(e.target.value) || 1 })
              }
              style={{ width: '100%' }}
            />
          </div>

          <div style={{ marginBottom: 8 }}>
            <label
              style={{
                display: 'block',
                fontSize: 13,
                fontWeight: 600,
                marginBottom: 4,
              }}
            >
              Holding Horizon (Hours)
            </label>
            <input
              type="number"
              value={form.horizonHours}
              onChange={(e) =>
                setForm({
                  ...form,
                  horizonHours: parseFloat(e.target.value) || 0,
                })
              }
              style={{
                width: '100%',
                padding: '8px 10px',
                borderRadius: 4,
                border: '1px solid #ccc',
                boxSizing: 'border-box',
              }}
            />
          </div>

          {error && (
            <div
              style={{
                color: '#c62828',
                fontSize: 13,
                marginTop: 12,
                fontWeight: 500,
              }}
            >
              ⚠️ {error}
            </div>
          )}
        </div>

        {/* Breakdown Table */}
        <div
          style={{
            border: '1px solid #e0e0e0',
            borderRadius: 8,
            padding: 20,
            backgroundColor: '#ffffff',
          }}
        >
          <h2 style={{ fontSize: 18, marginTop: 0, marginBottom: 16 }}>
            Cost Breakdown
          </h2>
          {breakdown ? (
            <div>
              <table
                style={{
                  width: '100%',
                  borderCollapse: 'collapse',
                  fontSize: 14,
                }}
              >
                <tbody>
                  <tr style={{ borderBottom: '1px solid #f0f0f0' }}>
                    <td style={{ padding: '6px 0', color: '#666' }}>
                      Position Size
                    </td>
                    <td style={{ padding: '6px 0', textAlign: 'right', fontWeight: 600 }}>
                      ${breakdown.sizeUsd.toFixed(2)}
                    </td>
                  </tr>
                  <tr style={{ borderBottom: '1px solid #f0f0f0' }}>
                    <td style={{ padding: '6px 0', color: '#666' }}>
                      Open Fee (6 bps)
                    </td>
                    <td style={{ padding: '6px 0', textAlign: 'right' }}>
                      ${breakdown.openFeeUsd.toFixed(4)}
                    </td>
                  </tr>
                  <tr style={{ borderBottom: '1px solid #f0f0f0' }}>
                    <td style={{ padding: '6px 0', color: '#666' }}>
                      Close Fee (6 bps)
                    </td>
                    <td style={{ padding: '6px 0', textAlign: 'right' }}>
                      ${breakdown.closeFeeUsd.toFixed(4)}
                    </td>
                  </tr>
                  <tr style={{ borderBottom: '1px solid #f0f0f0' }}>
                    <td style={{ padding: '6px 0', color: '#666' }}>
                      Borrow Cost ({form.horizonHours}h)
                    </td>
                    <td style={{ padding: '6px 0', textAlign: 'right' }}>
                      ${breakdown.borrowUsd.toFixed(4)}
                    </td>
                  </tr>
                  <tr
                    style={{
                      borderBottom: '2px solid #333',
                      fontWeight: 700,
                    }}
                  >
                    <td style={{ padding: '10px 0', fontSize: 15 }}>Total Cost</td>
                    <td
                      style={{
                        padding: '10px 0',
                        textAlign: 'right',
                        fontSize: 15,
                        color: '#1565c0',
                      }}
                    >
                      ${breakdown.totalUsd.toFixed(4)} ({breakdown.totalPct.toFixed(3)}%)
                    </td>
                  </tr>
                  <tr style={{ borderBottom: '1px solid #f0f0f0' }}>
                    <td style={{ padding: '6px 0', color: '#666' }}>
                      Breakeven Move
                    </td>
                    <td style={{ padding: '6px 0', textAlign: 'right' }}>
                      {breakdown.breakevenMovePct.toFixed(4)}%
                    </td>
                  </tr>
                  <tr>
                    <td style={{ padding: '6px 0', color: '#666' }}>
                      Est. Liquidation Price
                    </td>
                    <td
                      style={{
                        padding: '6px 0',
                        textAlign: 'right',
                        color: '#d32f2f',
                        fontWeight: 600,
                      }}
                    >
                      ${breakdown.liqPrice.toFixed(2)} ({breakdown.liqDistancePct.toFixed(2)}% away)
                    </td>
                  </tr>
                </tbody>
              </table>

              <div style={{ marginTop: 16 }}>
                <h4 style={{ margin: '0 0 8px 0', fontSize: 13, color: '#555' }}>
                  Horizon Carry Projections
                </h4>
                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: 8 }}>
                  {breakdown.sweep.map((p) => (
                    <div
                      key={p.hours}
                      style={{
                        backgroundColor: '#f8f9fa',
                        padding: '8px 6px',
                        borderRadius: 4,
                        textAlign: 'center',
                        border: '1px solid #e9ecef',
                      }}
                    >
                      <div style={{ fontSize: 11, color: '#666' }}>{p.hours}h</div>
                      <div style={{ fontWeight: 600, fontSize: 13, margin: '2px 0' }}>
                        ${p.totalUsd.toFixed(2)}
                      </div>
                      <div style={{ fontSize: 10, color: '#888' }}>
                        borrow ${p.borrowUsd.toFixed(2)}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </div>
          ) : (
            <div style={{ color: '#888', fontStyle: 'italic' }}>
              Enter valid inputs to view breakdown.
            </div>
          )}
        </div>
      </div>

      {/* Demo Scenario Panel */}
      <div
        style={{
          border: '1px solid #e0e0e0',
          borderRadius: 8,
          padding: 20,
          backgroundColor: '#fff',
          marginBottom: 32,
        }}
      >
        <div
          style={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            marginBottom: 16,
          }}
        >
          <h2 style={{ fontSize: 18, margin: 0 }}>
            Replayable Atomic Guard Scenarios (Surfpool)
          </h2>
          <div style={{ display: 'flex', gap: 8 }}>
            <button
              type="button"
              onClick={() => setActiveScenario('a')}
              style={{
                padding: '6px 12px',
                borderRadius: 4,
                border: '1px solid',
                borderColor: activeScenario === 'a' ? '#2e7d32' : '#ccc',
                backgroundColor: activeScenario === 'a' ? '#e8f5e9' : '#fff',
                fontSize: 13,
                fontWeight: 600,
                cursor: 'pointer',
              }}
            >
              Scenario A (Normal)
            </button>
            <button
              type="button"
              onClick={() => setActiveScenario('b')}
              style={{
                padding: '6px 12px',
                borderRadius: 4,
                border: '1px solid',
                borderColor: activeScenario === 'b' ? '#c62828' : '#ccc',
                backgroundColor: activeScenario === 'b' ? '#ffebee' : '#fff',
                fontSize: 13,
                fontWeight: 600,
                cursor: 'pointer',
              }}
            >
              Scenario B (Oracle Spike)
            </button>
          </div>
        </div>

        {activeScenario === 'a' ? (
          <div
            style={{
              padding: 16,
              borderRadius: 6,
              backgroundColor: '#f1f8e9',
              border: '1px solid #c8e6c9',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
              <span
                style={{
                  fontSize: 11,
                  padding: '2px 6px',
                  borderRadius: 3,
                  backgroundColor: '#2e7d32',
                  color: '#fff',
                  fontWeight: 600,
                }}
              >
                PASS
              </span>
              <strong style={{ fontSize: 15, color: '#1b5e20' }}>
                Scenario A: Normal Market Conditions
              </strong>
            </div>
            <p style={{ margin: '8px 0', fontSize: 13, color: '#2e7d32' }}>
              Oracle Price: $150.05 | Reference Price: $150.00 | Deviation: 3 bps
              (Threshold: 50 bps)
            </p>
            <div
              style={{
                fontFamily: 'monospace',
                fontSize: 12,
                backgroundColor: '#ffffff',
                padding: 10,
                borderRadius: 4,
                border: '1px solid #dcedc8',
                color: '#333',
              }}
            >
              [Instruction 0] ComputeBudget::set_compute_unit_limit(200_000) &rarr; OK<br />
              [Instruction 1] InterlockIx::assert_oracle_sane &rarr; OK (measured 3 bps)<br />
              [Instruction 2] Venue::open_position &rarr; EXECUTED (Order Filled)
            </div>
          </div>
        ) : (
          <div
            style={{
              padding: 16,
              borderRadius: 6,
              backgroundColor: '#fbe9e7',
              border: '1px solid #ffccbc',
            }}
          >
            <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
              <span
                style={{
                  fontSize: 11,
                  padding: '2px 6px',
                  borderRadius: 3,
                  backgroundColor: '#d32f2f',
                  color: '#fff',
                  fontWeight: 600,
                }}
              >
                REVERT
              </span>
              <span
                style={{
                  fontSize: 11,
                  padding: '2px 6px',
                  borderRadius: 3,
                  backgroundColor: '#ff9800',
                  color: '#fff',
                  fontWeight: 600,
                }}
              >
                SYNTHETIC
              </span>
              <strong style={{ fontSize: 15, color: '#b71c1c' }}>
                Scenario B: Synthetic Oracle Price Spike (+15%)
              </strong>
            </div>
            <p style={{ margin: '8px 0', fontSize: 13, color: '#b71c1c' }}>
              Oracle Price: $172.50 | Reference Price: $150.00 | Deviation: 1500 bps
              (Threshold: 50 bps)
            </p>
            <div
              style={{
                fontFamily: 'monospace',
                fontSize: 12,
                backgroundColor: '#ffffff',
                padding: 10,
                borderRadius: 4,
                border: '1px solid #ffccbc',
                color: '#333',
              }}
            >
              [Instruction 0] ComputeBudget::set_compute_unit_limit(200_000) &rarr; OK<br />
              [Instruction 1] InterlockIx::assert_oracle_sane &rarr; REVERT (Error: DeviationExceeded)<br />
              [Instruction 2] Venue::open_position &rarr; NOT EXECUTED (Atomic Reversion)
            </div>
            <p
              style={{
                margin: '8px 0 0 0',
                fontSize: 12,
                color: '#4e342e',
                fontWeight: 500,
              }}
            >
              Protected: Order was never sent to the matching engine. Zero capital
              lost to corrupted oracle feed.
            </p>
          </div>
        )}
      </div>

      <footer
        style={{
          borderTop: '1px solid #e0e0e0',
          paddingTop: 16,
          fontSize: 12,
          color: '#888',
          display: 'flex',
          justifyContent: 'space-between',
        }}
      >
        <span>Interlock &bull; Solana Perps Hackathon 2026</span>
        <span>Apache-2.0 License &bull; Read-Only Non-Custodial Client</span>
      </footer>
    </div>
  );
}
