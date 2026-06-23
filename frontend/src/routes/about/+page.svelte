<svelte:head>
  <title>About — Civitas</title>
</svelte:head>

<section class="mx-auto max-w-civic px-5 pb-20 pt-14 sm:px-10">
  <div class="mb-3.5 font-mono text-[11px] uppercase tracking-[0.2em] text-ink-400">About</div>
  <h1
    class="font-serif text-[clamp(40px,5.4vw,60px)] font-semibold leading-[1.04] tracking-[-0.015em]"
  >
    About Civitas
  </h1>

  <div class="prose-civic mt-8 text-[17px]">
    <p>
      Civitas is an open-source platform for direct democracy with optional per-topic delegation.
      The first version is intentionally minimal: a small, working tool that demonstrates the
      mechanism end-to-end. Larger ambitions — cryptographic verification, national e-ID
      integration, candidate commitment systems — are documented in the
      <a href="https://github.com/zarathustracode/civitas/blob/main/docs/roadmap.md">roadmap</a>
      but deliberately deferred.
    </p>

    <h2>Governance</h2>
    <p>
      Civitas is currently founder-controlled, transitioning in four phases to govern itself on its
      own platform. Phase 1 — founder makes decisions, contributors propose changes. Phase 2 —
      operational decisions migrate onto the platform. Phase 3 — architectural decisions migrate.
      Phase 4 — the founder's procedural authority ends.
    </p>
    <p>
      The full plan and the founder's commitments are in
      <a href="https://github.com/zarathustracode/civitas/blob/main/GOVERNANCE.md">GOVERNANCE.md</a
      >.
    </p>

    <h2>Source</h2>
    <p>
      Civitas is licensed under
      <a href="https://www.gnu.org/licenses/agpl-3.0.html">AGPL-3.0</a>
      so any networked deployment of a modified version must publish its source. This is the civic commons
      made explicit in the license.
    </p>
    <p>
      Source: <a href="https://github.com/zarathustracode/civitas" rel="noopener noreferrer"
        >github.com/zarathustracode/civitas</a
      >
    </p>

    <h2>Reporting issues</h2>
    <p>
      Bugs and proposals: open an issue on the GitHub repository. Security vulnerabilities: see
      <a href="https://github.com/zarathustracode/civitas/blob/main/SECURITY.md">SECURITY.md</a>
      — do not file public issues for security problems.
    </p>
  </div>
</section>
