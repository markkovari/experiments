import React, { useState } from 'react';
import './Calculator.css';

const Calculator = () => {
  const [number, setNumber] = useState('');
  const [result, setResult] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState(null);

  const API_URL = process.env.REACT_APP_API_URL || 'http://localhost:8080';

  const calculateFactorial = async () => {
    if (!number || isNaN(number) || parseInt(number) < 0) {
      setError('Please enter a valid non-negative number');
      return;
    }

    setLoading(true);
    setError(null);
    setResult(null);

    try {
      const response = await fetch(`${API_URL}/calculate`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          number: parseInt(number),
        }),
      });

      if (!response.ok) {
        throw new Error('Failed to trigger calculation');
      }

      const data = await response.json();

      if (data.status === 'error') {
        setError(data.error || 'Calculation failed');
      } else {
        setResult({
          value: data.result,
          requestId: data.request_id,
        });
      }
    } catch (err) {
      setError(err.message || 'Failed to calculate factorial');
    } finally {
      setLoading(false);
    }
  };

  const handleSubmit = (e) => {
    e.preventDefault();
    calculateFactorial();
  };

  return (
    <div className="calculator">
      <div className="calculator-card">
        <h1 className="title">Distributed Factorial Calculator</h1>
        <p className="subtitle">
          Powered by Windmill, NATS, SurrealDB & Go Workers
        </p>

        <form onSubmit={handleSubmit} className="calculator-form">
          <div className="input-group">
            <label htmlFor="number">Enter a number:</label>
            <input
              id="number"
              type="number"
              value={number}
              onChange={(e) => setNumber(e.target.value)}
              placeholder="e.g., 10"
              min="0"
              disabled={loading}
            />
          </div>

          <button type="submit" disabled={loading} className="calculate-btn">
            {loading ? 'Calculating...' : 'Calculate Factorial'}
          </button>
        </form>

        {error && (
          <div className="error-message">
            <strong>Error:</strong> {error}
          </div>
        )}

        {result && (
          <div className="result-card">
            <h2>Result</h2>
            <div className="result-content">
              <div className="result-row">
                <span className="label">Number:</span>
                <span className="value">{number}</span>
              </div>
              <div className="result-row">
                <span className="label">Factorial:</span>
                <span className="value result-value">{result.value}</span>
              </div>
              <div className="result-row">
                <span className="label">Request ID:</span>
                <span className="value request-id">{result.requestId}</span>
              </div>
            </div>
          </div>
        )}

        <div className="info-card">
          <h3>How it works:</h3>
          <ol>
            <li>Frontend sends request to Windmill workflow</li>
            <li>Windmill publishes message to NATS queue</li>
            <li>Multiple Go workers process recursively</li>
            <li>Results cached in SurrealDB</li>
            <li>All calculations logged for analytics</li>
          </ol>
        </div>
      </div>
    </div>
  );
};

export default Calculator;
