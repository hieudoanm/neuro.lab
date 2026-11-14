import type { Metadata, NextPage } from 'next';
import type { FC } from 'react';
import CTA from '@/components/molecules/CTA';
import Features, { type Feature } from '@/components/molecules/Features';
import Hero from '@/components/molecules/Hero';
import SectionHead from '@/components/molecules/SectionHead';
import Terminal from '@/components/molecules/Terminal';
import {
  LuActivity,
  LuBot,
  LuBoxes,
  LuClipboardCheck,
  LuDownload,
  LuList,
  LuScan,
} from 'react-icons/lu';

export const metadata: Metadata = {
  alternates: { canonical: '/' },
};

const GITHUB = 'https://github.com/hieudoanm/neuro.lab';

const FEATURES: Feature[] = [
  {
    icon: LuActivity,
    title: 'EEG pipelines',
    body: 'BrainVision, EDF+, and NPY input. Filtering, re-referencing, baseline correction, event-locked epoching, FastICA with topographies, and SSP.',
  },
  {
    icon: LuScan,
    title: 'MEG pipelines',
    body: 'FIF, CTF, KIT, BIDS, and OPM input. Compensation grades, FIF projectors, synthetic gradiometers, ICA, SSP, epoching and evoked responses — plus a low-order SSS approximation.',
  },
  {
    icon: LuBoxes,
    title: 'MRI pipelines',
    body: 'NIfTI and DICOM volumes with multiplanar slices, MIP, statistics, bias-field correction, heuristic skull stripping, and rigid/affine registration.',
  },
  {
    icon: LuList,
    title: 'Reproducible pipelines',
    body: 'Pipelines are plain JSON node lists — filter, rereference, ICA, SSP, epoch, register — saved as named presets. Every run records its parameters and produces a manifest.',
  },
  {
    icon: LuDownload,
    title: 'Interoperable exports',
    body: 'NPY, CSV, and EDF for signals; FIF output validated against MNE; BIDS datasets; NIfTI volumes; DICOM converted to NIfTI with spatial metadata preserved.',
  },
  {
    icon: LuClipboardCheck,
    title: 'Versioned, resumable storage',
    body: 'Chunked uploads with SHA-256 integrity verification, dataset versions, experiment records, and artifact provenance — all in a local workspace database.',
  },
  {
    icon: LuBot,
    title: 'MCP server',
    body: 'neuro mcp lets agents inspect, validate, convert, and export EEG, MEG, and MRI recordings through typed tools — fully local, over stdio.',
  },
];

const PIPELINE_JSON = [
  '{',
  '  "nodes": [',
  '    { "type": "bandpass", "params": { "l_freq_hz": 1, "h_freq_hz": 40 } },',
  '    { "type": "ica", "params": { "topographies": true } },',
  '    { "type": "epoch", "params": { "tmin_s": -0.2, "tmax_s": 0.5 } }',
  '  ]',
  '}',
];

const DEMO_TERMINALS: {
  caption: string;
  lines: string[];
  wide?: boolean;
}[] = [
  { caption: 'eeg-pipeline.json', wide: true, lines: PIPELINE_JSON },
  {
    caption: 'meg-pipeline.json',
    lines: [
      '{',
      '  "nodes": [',
      '    { "type": "bandpass", "params": { "l_freq_hz": 1, "h_freq_hz": 40 } },',
      '    { "type": "sss", "params": { "l_in": 4, "l_out": 2 } },',
      '    { "type": "evoked", "params": {} }',
      '  ]',
      '}',
    ],
  },
  {
    caption: 'Workspace layout',
    lines: [
      '$ ls crates',
      'neuro-core   shared numerics, ICA',
      'neuro-eeg    EEG readers + pipelines',
      'neuro-meg    MEG readers + pipelines',
      'neuro-mri    MRI volumes + DICOM',
      'neuro-store  workspace persistence',
    ],
  },
];

const STEPS = [
  {
    title: 'Import',
    body: 'Point the app at a recording. Neuro Lab resolves companion files — channel manifests, event markers, digitization points — and validates the file set before analysis.',
    snippet: 'recording.eeg + .vhdr + .vmrk',
  },
  {
    title: 'Compose',
    body: 'Chain pipeline nodes in the UI or paste JSON directly. Filter, reject artifacts, epoch, register — save the chain as a preset and rerun it on any compatible dataset.',
    snippet: 'pipeline.json → named preset',
  },
  {
    title: 'Inspect and export',
    body: 'Browse signal windows, spectra, epochs, ICA components, and MRI slices. Export to NPY, CSV, EDF, FIF, BIDS, or NIfTI — every artifact carries its provenance record.',
    snippet: 'artifacts/ with manifest',
  },
];

const INSTALL_LINES = [
  '$ git clone https://github.com/hieudoanm/neuro.lab',
  '$ cd neuro.lab',
  '$ pnpm install',
  '$ pnpm tauri:dev     # desktop app in dev mode',
  '$ pnpm tauri:build   # release bundles',
  '$ cargo test --workspace   # 24 test suites',
];

const StepCard: FC<{ step: (typeof STEPS)[number]; index: number }> = ({
  step,
  index,
}) => (
  <article className="card border-base-300 bg-base-100 rounded-2xl border p-6">
    <span className="text-accent mb-2 block font-mono text-sm font-semibold">
      {String(index + 1).padStart(2, '0')}
    </span>
    <h3 className="text-base-content m-0 mb-2 text-lg font-bold">
      {step.title}
    </h3>
    <p className="text-base-content/60 text-sm">{step.body}</p>
    <code className="border-base-300 bg-base-200 text-base-content/60 mt-4 block overflow-x-auto rounded-lg border px-3 py-2 font-mono text-xs">
      {step.snippet}
    </code>
  </article>
);

const HomePage: NextPage = () => (
  <main>
    <Hero
      title="EEG, MEG, and MRI — processed on your own machine."
      description="Neuro Lab is a local-first neuroimaging workbench: a Tauri desktop app backed by a Rust workspace of processing crates. Recordings never leave your machine — no server, no cloud, no accounts."
      primary={{ label: 'Get started', href: '#install' }}
      secondary={{ label: 'See pipelines', href: '#demo' }}
      stats={[
        { value: '3', label: 'modalities' },
        { value: '10+', label: 'input formats' },
        { value: '0', label: 'network calls' },
      ]}>
      <Terminal lines={PIPELINE_JSON} />
    </Hero>

    <Features
      title="One workspace for three modalities"
      description="EEG, MEG, and MRI datasets live side by side in a single local workspace with projects, experiments, and full provenance."
      features={FEATURES}
    />

    <section id="demo" className="bg-base-200 py-20">
      <div className="mx-auto w-full max-w-6xl px-5">
        <SectionHead
          title="Pipelines as data"
          description="No hidden steps — a pipeline is a JSON list of nodes you can read, save, and share. These are the same definitions the app runs."
        />
        <div className="grid grid-cols-1 gap-6 lg:grid-cols-2">
          {DEMO_TERMINALS.map((t) => (
            <Terminal
              key={t.caption}
              lines={t.lines}
              className={t.wide ? 'lg:col-span-2' : ''}
            />
          ))}
        </div>
      </div>
    </section>

    <section id="how-it-works" className="py-20">
      <div className="mx-auto w-full max-w-6xl px-5">
        <SectionHead
          title="How it works"
          description="From raw recording to published-format export in three steps."
        />
        <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
          {STEPS.map((step, i) => (
            <StepCard key={step.title} step={step} index={i} />
          ))}
        </div>
      </div>
    </section>

    <section id="install" className="bg-base-200 py-20">
      <div className="mx-auto w-full max-w-3xl px-5">
        <SectionHead
          title="Build from source"
          description="Requires Node.js + pnpm and Rust. Everything compiles locally — there is no backend service to deploy."
        />
        <Terminal lines={INSTALL_LINES} />
        <p className="text-base-content/60 mt-6 text-center text-sm">
          See the{' '}
          <a
            href={`${GITHUB}#readme`}
            target="_blank"
            rel="noreferrer"
            className="text-primary font-semibold">
            README
          </a>{' '}
          for prerequisites and crate details.
        </p>
      </div>
    </section>

    <CTA
      title="Ready to process recordings?"
      description="EEG, MEG, and MRI — all local, all reproducible."
      primary={{ label: 'View on GitHub', href: GITHUB, external: true }}
      secondary={{
        label: 'Read the README',
        href: `${GITHUB}#readme`,
        external: true,
      }}
    />
  </main>
);

export default HomePage;
