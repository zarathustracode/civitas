// Read-path latency budget for the API, run with k6 (https://k6.io).
//
//   k6 run backend/perf/reads.js
//
// Drives the endpoints behind the public pages at a steady request rate
// and fails if any endpoint's P99 exceeds the budget, or if any request
// fails. Run it against a release build: debug-build latency says nothing
// about production.
//
// Setup signs in as the seeded users and grows the seed data into a docket
// of a few dozen proposals with votes, so the list and tally endpoints do
// real work. It needs the seed script to have run, and the API started
// with rate limits high enough for one client (see .github/workflows/perf.yml).

import http from 'k6/http';
import { check, fail } from 'k6';

const API = __ENV.API_BASE_URL || 'http://127.0.0.1:8080';
const P99_MS = Number(__ENV.P99_BUDGET_MS || 50);
const SEED_PASSWORD = 'civitas-dev-pw-v1';
const FIXTURE_PROPOSALS = 40;

const ENDPOINTS = [
  'health',
  'summaries',
  'proposals_voting',
  'proposal',
  'tally',
  'comments',
  'audit',
  'topics',
  'topic_stats'
];

export const options = {
  scenarios: {
    reads: {
      executor: 'constant-arrival-rate',
      rate: Number(__ENV.RATE || 100),
      timeUnit: '1s',
      duration: __ENV.DURATION || '30s',
      preAllocatedVUs: 20,
      maxVUs: 100
    }
  },
  thresholds: Object.fromEntries([
    ['http_req_failed{scenario:reads}', ['rate==0']],
    ...ENDPOINTS.map((e) => [`http_req_duration{endpoint:${e}}`, [`p(99)<${P99_MS}`]])
  ]),
  summaryTrendStats: ['med', 'p(95)', 'p(99)', 'max']
};

function login(email) {
  const jar = http.cookieJar();
  jar.clear(API);
  const res = http.post(`${API}/auth/login`, JSON.stringify({ email, password: SEED_PASSWORD }), {
    headers: { 'Content-Type': 'application/json' }
  });
  if (res.status !== 200) fail(`login as ${email}: ${res.status} ${res.body}`);
}

function post(path, body) {
  const res = http.post(`${API}${path}`, JSON.stringify(body), {
    headers: { 'Content-Type': 'application/json' }
  });
  if (res.status >= 300) fail(`POST ${path}: ${res.status} ${res.body}`);
  return res.json();
}

export function setup() {
  const topic = http.get(`${API}/topics/demo-policy`);
  if (topic.status !== 200) fail('seed topic demo-policy missing; run the seed script');
  const topicId = topic.json('id');

  login('dave@example.com');
  const now = new Date();
  const ends = new Date(now.getTime() + 7 * 24 * 60 * 60 * 1000);
  const created = [];
  for (let i = 0; i < FIXTURE_PROPOSALS; i++) {
    const p = post('/proposals', {
      topic_id: topicId,
      title: `Performance fixture ${i + 1}`,
      summary: 'Part of the read-path latency budget run.',
      body: 'Created by backend/perf/reads.js.'
    });
    // Leave a quarter in draft, a quarter in deliberation, the rest voting.
    if (i % 4 !== 0) post(`/proposals/${p.id}/status`, { target: 'deliberation' });
    if (i % 4 >= 2) {
      post(`/proposals/${p.id}/status`, {
        target: 'voting',
        voting_starts_at: now.toISOString(),
        voting_ends_at: ends.toISOString()
      });
      created.push(p.id);
    }
  }

  const choices = ['yes', 'no', 'abstain'];
  ['alice@example.com', 'bob@example.com', 'carol@example.com'].forEach((email, u) => {
    login(email);
    created.forEach((id, i) => post(`/proposals/${id}/votes`, { choice: choices[(i + u) % 3] }));
  });
  http.cookieJar().clear(API);

  return { proposals: created, slug: 'demo-policy' };
}

let n = 0;

export default function (data) {
  const endpoint = ENDPOINTS[n % ENDPOINTS.length];
  const id = data.proposals[Math.floor(n / ENDPOINTS.length) % data.proposals.length];
  n++;
  const path = {
    health: '/health',
    summaries: '/proposals/summaries',
    proposals_voting: '/proposals?status=voting',
    proposal: `/proposals/${id}`,
    tally: `/proposals/${id}/tally`,
    comments: `/proposals/${id}/comments`,
    audit: `/proposals/${id}/audit`,
    topics: '/topics',
    topic_stats: `/topics/${data.slug}/stats`
  }[endpoint];
  const res = http.get(`${API}${path}`, { tags: { endpoint } });
  check(res, { 'status is 200': (r) => r.status === 200 });
}
