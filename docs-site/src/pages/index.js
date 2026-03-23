import React from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import styles from './index.module.css';

function Hero() {
  const {siteConfig} = useDocusaurusContext();
  return (
    <header className={clsx('hero hero--primary', styles.heroBanner)}>
      <div className="container">
        <h1 className="hero__title">{siteConfig.title}</h1>
        <p className="hero__subtitle">{siteConfig.tagline}</p>
        <p className={styles.heroDescription}>
          One CLI to scaffold reproducible, containerized dev environments with
          built-in AI context, curated skills, and structured work processes.
        </p>
        <div className={styles.buttons}>
          <Link className="button button--secondary button--lg" to="/docs/getting-started/installation">
            Get Started →
          </Link>
        </div>
      </div>
    </header>
  );
}

const features = [
  {
    title: 'One Command Setup',
    description: 'From zero to a fully configured dev environment with dev-box init. Container, AI context, skills, and theming — all scaffolded.',
  },
  {
    title: '13 Process Packages',
    description: 'Composable workflow packages: tracking, code, research, design, security, operations, and more. Pick what fits your project.',
  },
  {
    title: '83 Curated Skills',
    description: 'Vetted AI agent skills with progressive disclosure. Not skill slop — each skill is handcrafted with examples and reference files.',
  },
  {
    title: '17 Add-ons',
    description: 'Python, Rust, Node, LaTeX, Kubernetes, and more — each with per-tool version selection from curated lists.',
  },
];

function Feature({title, description}) {
  return (
    <div className={clsx('col col--6')}>
      <div className="padding-horiz--md padding-vert--md">
        <h3>{title}</h3>
        <p>{description}</p>
      </div>
    </div>
  );
}

function Features() {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {features.map((props, idx) => <Feature key={idx} {...props} />)}
        </div>
      </div>
    </section>
  );
}

function QuickStart() {
  return (
    <section className={styles.quickstart}>
      <div className="container">
        <h2>Quick Start</h2>
        <pre><code>{`curl -fsSL https://raw.githubusercontent.com/projectious-work/dev-box/main/install.sh | sh
dev-box init --name my-project --base debian
dev-box build && dev-box start`}</code></pre>
      </div>
    </section>
  );
}

export default function Home() {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout title="Home" description={siteConfig.tagline}>
      <Hero />
      <main>
        <Features />
        <QuickStart />
      </main>
    </Layout>
  );
}
