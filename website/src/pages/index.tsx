import type { ReactNode } from 'react';
import Link from '@docusaurus/Link';
import useBaseUrl from '@docusaurus/useBaseUrl';
import Layout from '@theme/Layout';
import styles from './index.module.css';

export default function Home(): ReactNode {
  return (
    <Layout
      title="The last API client you'll ever need"
      description="5MB. 50ms startup. Zero monthly fees. A native API client that respects your time, your privacy, and your sanity.">

      {/* Hero Section */}
      <section className={styles.hero}>
        <div className={styles.heroContainer}>
          <div className={styles.heroBadge}>
            <span className={styles.badgeDot}>●</span>
            <span>v0.2.0-beta — Open Source</span>
          </div>

          <h1 className={styles.heroTitle}>
            <span className={styles.line}>
              <span className={styles.strike}>500MB</span> → <span className={styles.accent}>5MB</span>
            </span>
            <span className={styles.line}>
              <span className={styles.strike}>$25/mo</span> → <span className={styles.accent}>$0</span>
            </span>
            <span className={styles.line}>
              <span className={styles.strike}>2 sec</span> → <span className={styles.accent}>50ms</span>
            </span>
          </h1>

          <p className={styles.heroSub}>
            Mercury is a native API client written in Rust. No Electron bloat. No cloud accounts.
            No subscription fees. Just <code>⌘+Enter</code> and your request is sent.
          </p>

          <div className={styles.heroCta}>
            <Link
              className={styles.btnPrimary}
              to="https://github.com/Harry-kp/mercury/releases">
              ↓ Download Free
            </Link>
            <Link
              className={styles.btnSecondary}
              to="/docs/getting-started">
              Read the Docs →
            </Link>
          </div>

          <div className={styles.metrics}>
            <div className={styles.metric}>
              <div className={styles.metricValue}>5MB</div>
              <div className={styles.metricLabel}>Binary Size</div>
            </div>
            <div className={styles.metric}>
              <div className={styles.metricValue}>&lt;50ms</div>
              <div className={styles.metricLabel}>Startup</div>
            </div>
            <div className={styles.metric}>
              <div className={styles.metricValue}>30MB</div>
              <div className={styles.metricLabel}>RAM Usage</div>
            </div>
            <div className={styles.metric}>
              <div className={styles.metricValue}>$0</div>
              <div className={styles.metricLabel}>Forever</div>
            </div>
          </div>

          {/* Hero Screenshot */}
          <div className={styles.heroScreenshot}>
            <img
              src={useBaseUrl('/img/screenshot.png')}
              alt="Mercury App - 3-column layout showing sidebar, request editor, and response panel"
              loading="lazy"
            />
          </div>
        </div>
      </section>

      {/* Manifesto Section */}
      <section className={styles.manifesto}>
        <div className={styles.manifestoContainer}>
          <div className={styles.manifestoLabel}>// The Philosophy</div>
          <h2 className={styles.manifestoTitle}>
            We said no to everything<br />that doesn't matter
          </h2>
          <p className={styles.manifestoText}>
            While Postman adds AI assistants, team collaboration, and 47 features you'll never use,
            <strong> we built the opposite</strong>. Mercury does one thing: send HTTP requests. Fast.
            <br /><br />
            Inspired by <strong>37signals</strong> and the Unix philosophy.
            No accounts. No cloud. No tracking. No bloat. Your data stays on your machine, in plain text files
            you can grep.
          </p>

          <div className={styles.principles}>
            <div className={styles.principle}>
              <div className={styles.principleIcon}>⚡</div>
              <h3>Native Performance</h3>
              <p>Written in Rust. Launches before Postman finishes checking for updates. No Electron, no web views, no memory hogs.</p>
            </div>
            <div className={styles.principle}>
              <div className={styles.principleIcon}>📁</div>
              <h3>Files, Not Databases</h3>
              <p><strong>Live Two-Way Sync.</strong> Edit in VS Code, run in Mercury. Change a file, it updates instantly. No import/export.</p>
            </div>
            <div className={styles.principle}>
              <div className={styles.principleIcon}>🔒</div>
              <h3>Truly Local</h3>
              <p>We don't have servers. We can't see your API keys even if we wanted to. Your secrets stay yours.</p>
            </div>
            <div className={styles.principle}>
              <div className={styles.principleIcon}>⌨️</div>
              <h3>Keyboard First</h3>
              <p>⌘+Enter to send. ⌘+S to save. ⌘+K to search. Your hands stay on the keyboard where they belong.</p>
            </div>
          </div>
        </div>
      </section>

      {/* Comparison Section */}
      <section className={styles.comparison}>
        <div className={styles.comparisonContainer}>
          <h2 className={styles.comparisonTitle}>The honest comparison</h2>
          <p className={styles.comparisonSub}>Numbers don't lie. Marketing does.</p>

          <div className={styles.comparisonTable}>
            <div className={styles.comparisonHeader}>
              <div></div>
              <div>Postman</div>
              <div>Insomnia</div>
              <div className={styles.mercuryCol}>Mercury</div>
            </div>
            <div className={styles.comparisonRow}>
              <div>App Size</div>
              <div className={styles.bad}>~500MB</div>
              <div className={styles.bad}>~400MB</div>
              <div className={styles.good}>5MB</div>
            </div>
            <div className={styles.comparisonRow}>
              <div>Startup Time</div>
              <div className={styles.bad}>3-5 seconds</div>
              <div className={styles.bad}>2-4 seconds</div>
              <div className={styles.good}>&lt;50ms</div>
            </div>
            <div className={styles.comparisonRow}>
              <div>Memory Usage</div>
              <div className={styles.bad}>300-800MB</div>
              <div className={styles.bad}>200-500MB</div>
              <div className={styles.good}>~30MB</div>
            </div>
            <div className={styles.comparisonRow}>
              <div>Price (Pro)</div>
              <div className={styles.bad}>$14-25/mo</div>
              <div className={styles.warn}>$5-18/mo</div>
              <div className={styles.good}>Free forever</div>
            </div>
            <div className={styles.comparisonRow}>
              <div>Account Required</div>
              <div className={styles.bad}>Yes</div>
              <div className={styles.bad}>Yes</div>
              <div className={styles.good}>No</div>
            </div>
            <div className={styles.comparisonRow}>
              <div>Open Source</div>
              <div className={styles.bad}>No</div>
              <div className={styles.warn}>Partially</div>
              <div className={styles.good}>100%</div>
            </div>
            <div className={styles.comparisonRow}>
              <div>Telemetry</div>
              <div className={styles.bad}>Yes</div>
              <div className={styles.bad}>Yes</div>
              <div className={styles.good}>None</div>
            </div>
            <div className={styles.comparisonRow}>
              <div>Works Offline</div>
              <div className={styles.warn}>Limited</div>
              <div className={styles.warn}>Limited</div>
              <div className={styles.good}>Always</div>
            </div>
          </div>
        </div>
      </section>

      {/* Code Section */}
      <section className={styles.codeSection}>
        <div className={styles.codeContainer}>
          <h2>Your requests. Your files.</h2>
          <p>No proprietary databases. Just plain text you can version control.</p>
          <pre className={styles.codeBlock}>
            <code>
              <span className={styles.comment}># ~/api-tests/users/get-user.http</span>{'\n\n'}
              <span className={styles.keyword}>GET</span> <span className={styles.string}>https://api.example.com/users/{'{{user_id}}'}</span>{'\n'}
              <span className={styles.key}>Authorization</span>: <span className={styles.value}>Bearer {'{{token}}'}</span>{'\n'}
              <span className={styles.key}>Accept</span>: <span className={styles.value}>application/json</span>{'\n\n'}
              <span className={styles.comment}># Variables loaded from .env files. Git-friendly. Greppable.</span>
            </code>
          </pre>
        </div>
      </section>

      {/* Download Section */}
      <section className={styles.download}>
        <div className={styles.downloadContainer}>
          <h2 className={styles.downloadTitle}>Get Mercury</h2>
          <p className={styles.downloadSub}>One download. No signup. No credit card.</p>

          <div className={styles.downloadCards}>
            <Link to="https://github.com/Harry-kp/mercury/releases" className={styles.downloadCard}>
              <div className={styles.downloadIcon}>🍎</div>
              <div className={styles.downloadPlatform}>macOS</div>
              <div className={styles.downloadMeta}>Universal • 5MB</div>
            </Link>
            <Link to="https://github.com/Harry-kp/mercury/releases" className={styles.downloadCard}>
              <div className={styles.downloadIcon}>⊞</div>
              <div className={styles.downloadPlatform}>Windows</div>
              <div className={styles.downloadMeta}>x64 • 5MB</div>
            </Link>
            <Link to="https://github.com/Harry-kp/mercury/releases" className={styles.downloadCard}>
              <div className={styles.downloadIcon}>🐧</div>
              <div className={styles.downloadPlatform}>Linux</div>
              <div className={styles.downloadMeta}>AppImage • 5MB</div>
            </Link>
          </div>

          <div className={styles.downloadAlt}>
            Or build from source:<br />
            <code className={styles.installCmd}>cargo install mercury</code>
          </div>
        </div>
      </section>
    </Layout>
  );
}
