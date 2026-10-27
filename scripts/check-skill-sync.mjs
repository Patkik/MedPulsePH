import { readdir, readFile } from 'node:fs/promises';
import { join } from 'node:path';

const roots = ['.agents/skills', '.claude/skills'];

async function listSkills(root) {
  const entries = await readdir(root, { withFileTypes: true });
  const names = await Promise.all(entries.filter((entry) => entry.isDirectory()).map(async (entry) => {
    try {
      await readFile(join(root, entry.name, 'SKILL.md'));
      return entry.name;
    } catch (error) {
      if (error.code === 'ENOENT') return null;
      throw error;
    }
  }));
  return names.filter(Boolean).sort();
}

const [agentSkills, claudeSkills] = await Promise.all(roots.map(listSkills));
if (JSON.stringify(agentSkills) !== JSON.stringify(claudeSkills)) {
  throw new Error('Workspace skill directories differ between .agents and .claude.');
}

for (const skill of agentSkills) {
  const [agent, claude] = await Promise.all(
    roots.map((root) => readFile(join(root, skill, 'SKILL.md'), 'utf8')),
  );
  if (agent.replace(/\r\n/g, '\n') !== claude.replace(/\r\n/g, '\n')) {
    throw new Error(`Workspace skill copies differ: ${skill}`);
  }
}

const [agents, claude] = await Promise.all(['AGENTS.md', 'CLAUDE.md'].map((path) => readFile(path, 'utf8')));
if (agents.replace(/\r\n/g, '\n') !== claude.replace(/\r\n/g, '\n')) {
  throw new Error('AGENTS.md and CLAUDE.md differ.');
}

console.log(`Verified ${agentSkills.length} mirrored skill instructions and project instructions.`);
