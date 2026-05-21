import { BaseChatMessageHistory } from "@langchain/core/chat_history";
import { AIMessage, HumanMessage } from "@langchain/core/messages";
import minigraf from "minigraf";

const { MiniGrafDb } = minigraf;

export const EXPECTED_OUTPUT = `Human: Remember that Minigraf stores agent memory.
AI: Got it. I will use Minigraf-backed chat history.`;

function datalogString(value) {
  return JSON.stringify(value);
}

function sessionSlug(value) {
  return value.replace(/[^A-Za-z0-9_-]+/g, "-").replace(/^-+|-+$/g, "") || "default";
}

function messageRole(message) {
  if (message instanceof HumanMessage) {
    return "human";
  }
  if (message instanceof AIMessage) {
    return "ai";
  }
  return message.getType();
}

function roleToMessage(role, content) {
  if (role === "human") {
    return new HumanMessage(content);
  }
  if (role === "ai") {
    return new AIMessage(content);
  }
  throw new Error(`unsupported message role: ${role}`);
}

export class MinigrafChatMessageHistory extends BaseChatMessageHistory {
  constructor({ sessionId, db = MiniGrafDb.inMemory() }) {
    super();
    this.sessionId = sessionId;
    this.sessionSlug = sessionSlug(sessionId);
    this.db = db;
  }

  async getMessages() {
    const session = datalogString(this.sessionId);
    const result = JSON.parse(
      this.db.execute(`(query [:find ?m ?index ?role ?content
                               :where [?m :message/session ${session}]
                                      [?m :message/index ?index]
                                      [?m :message/role ?role]
                                      [?m :message/content ?content]])`),
    );
    return result.results
      .sort((left, right) => left[1] - right[1])
      .map(([, , role, content]) => roleToMessage(role, content));
  }

  async addMessages(messages) {
    const start = (await this.getMessages()).length;
    const facts = messages.flatMap((message, offset) => {
      const index = start + offset;
      const entity = `:message/${this.sessionSlug}-${index}`;
      return [
        `[${entity} :message/session ${datalogString(this.sessionId)}]`,
        `[${entity} :message/index ${index}]`,
        `[${entity} :message/role ${datalogString(messageRole(message))}]`,
        `[${entity} :message/content ${datalogString(String(message.content))}]`,
      ];
    });
    if (facts.length > 0) {
      this.db.execute(`(transact [${facts.join(" ")}])`);
    }
  }

  async addMessage(message) {
    await this.addMessages([message]);
  }

  async clear() {
    const session = datalogString(this.sessionId);
    const result = JSON.parse(
      this.db.execute(`(query [:find ?m ?index ?role ?content
                               :where [?m :message/session ${session}]
                                      [?m :message/index ?index]
                                      [?m :message/role ?role]
                                      [?m :message/content ?content]])`),
    );
    const facts = result.results.flatMap(([messageId, index, role, content]) => {
      const entity = `#uuid "${messageId}"`;
      return [
        `[${entity} :message/session ${session}]`,
        `[${entity} :message/index ${index}]`,
        `[${entity} :message/role ${datalogString(role)}]`,
        `[${entity} :message/content ${datalogString(content)}]`,
      ];
    });
    if (facts.length > 0) {
      this.db.execute(`(retract [${facts.join(" ")}])`);
    }
  }
}

export async function demo() {
  const history = new MinigrafChatMessageHistory({ sessionId: "agent-demo" });
  await history.addMessages([
    new HumanMessage("Remember that Minigraf stores agent memory."),
    new AIMessage("Got it. I will use Minigraf-backed chat history."),
  ]);

  for (const message of await history.getMessages()) {
    const label = message instanceof HumanMessage ? "Human" : "AI";
    console.log(`${label}: ${message.content}`);
  }
}

if (import.meta.url === `file://${process.argv[1]}`) {
  await demo();
}
