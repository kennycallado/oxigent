import type { Plugin } from "@opencode-ai/plugin"

async function readText(path: string): Promise<string | null> {
  try {
    return (await Bun.file(path).text()).trim()
  } catch {
    return null
  }
}

export const AgentsGuard: Plugin = async (ctx) => {
  const agentsMd = await readText(`${ctx.directory}/AGENTS.md`)

  await ctx.client.app.log({
    body: {
      service: "agents-guard",
      level: "info",
      message: agentsMd
        ? `Loaded AGENTS.md (${agentsMd.length} chars)`
        : "No AGENTS.md found",
    },
  })

  return {
    "experimental.session.compacting": async (_input, output) => {
      if (!agentsMd) return

      output.context.push(
        `## AGENTS.md (MUST PRESERVE — PROJECT RULES)\n\n${agentsMd}`
      )
    },
  }
}
