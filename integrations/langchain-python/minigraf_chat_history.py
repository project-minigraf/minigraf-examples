from __future__ import annotations

import json
import re
import tempfile
from pathlib import Path
from typing import Sequence

from langchain_core.chat_history import BaseChatMessageHistory
from langchain_core.messages import AIMessage, BaseMessage, HumanMessage
from minigraf import MiniGrafDb


EXPECTED_OUTPUT = """Human: Remember that Minigraf stores agent memory.
AI: Got it. I will use Minigraf-backed chat history."""


def _datalog_string(value: str) -> str:
    return json.dumps(value)


def _session_slug(value: str) -> str:
    slug = re.sub(r"[^A-Za-z0-9_-]+", "-", value).strip("-")
    return slug or "default"


def _message_to_role(message: BaseMessage) -> str:
    if isinstance(message, HumanMessage):
        return "human"
    if isinstance(message, AIMessage):
        return "ai"
    return message.type


def _role_to_message(role: str, content: str) -> BaseMessage:
    if role == "human":
        return HumanMessage(content=content)
    if role == "ai":
        return AIMessage(content=content)
    raise ValueError(f"unsupported message role: {role}")


class MinigrafChatMessageHistory(BaseChatMessageHistory):
    def __init__(self, session_id: str, path: str | Path | None = None) -> None:
        self.session_id = session_id
        self._session_slug = _session_slug(session_id)
        self._db = MiniGrafDb.open(str(path)) if path else MiniGrafDb.open_in_memory()

    @property
    def messages(self) -> list[BaseMessage]:
        session = _datalog_string(self.session_id)
        result = json.loads(
            self._db.execute(
                f"""(query [:find ?m ?index ?role ?content
                            :where [?m :message/session {session}]
                                   [?m :message/index ?index]
                                   [?m :message/role ?role]
                                   [?m :message/content ?content]])"""
            )
        )
        rows = sorted(result["results"], key=lambda row: row[1])
        return [_role_to_message(role, content) for _, _, role, content in rows]

    def add_messages(self, messages: Sequence[BaseMessage]) -> None:
        start = len(self.messages)
        facts = []
        for offset, message in enumerate(messages):
            index = start + offset
            entity = f":message/{self._session_slug}-{index}"
            role = _datalog_string(_message_to_role(message))
            content = _datalog_string(str(message.content))
            session = _datalog_string(self.session_id)
            facts.extend(
                [
                    f"[{entity} :message/session {session}]",
                    f"[{entity} :message/index {index}]",
                    f"[{entity} :message/role {role}]",
                    f"[{entity} :message/content {content}]",
                ]
            )
        if facts:
            self._db.execute(f"(transact [{' '.join(facts)}])")

    def clear(self) -> None:
        session = _datalog_string(self.session_id)
        result = json.loads(
            self._db.execute(
                f"""(query [:find ?m ?index ?role ?content
                            :where [?m :message/session {session}]
                                   [?m :message/index ?index]
                                   [?m :message/role ?role]
                                   [?m :message/content ?content]])"""
            )
        )
        facts = []
        for message_id, index, role, content in result["results"]:
            entity = f'#uuid "{message_id}"'
            facts.extend(
                [
                    f"[{entity} :message/session {session}]",
                    f"[{entity} :message/index {index}]",
                    f"[{entity} :message/role {_datalog_string(role)}]",
                    f"[{entity} :message/content {_datalog_string(content)}]",
                ]
            )
        if facts:
            self._db.execute(f"(retract [{' '.join(facts)}])")


def main() -> None:
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "langchain-memory.mg"
        history = MinigrafChatMessageHistory("agent-demo", path)
        history.add_messages(
            [
                HumanMessage(content="Remember that Minigraf stores agent memory."),
                AIMessage(content="Got it. I will use Minigraf-backed chat history."),
            ]
        )

        for message in history.messages:
            label = "Human" if isinstance(message, HumanMessage) else "AI"
            print(f"{label}: {message.content}")


if __name__ == "__main__":
    main()
