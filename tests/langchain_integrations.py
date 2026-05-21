from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def test_python_langchain_integration_pins_minigraf_1_1_1():
    requirements = (
        ROOT / "integrations" / "langchain-python" / "requirements.txt"
    ).read_text()

    assert "minigraf==1.1.1" in requirements
    assert "langchain-core" in requirements


def test_python_langchain_example_documents_expected_output():
    example = (
        ROOT / "integrations" / "langchain-python" / "minigraf_chat_history.py"
    ).read_text()

    assert "class MinigrafChatMessageHistory(BaseChatMessageHistory)" in example
    assert "Human: Remember that Minigraf stores agent memory." in example
    assert "AI: Got it. I will use Minigraf-backed chat history." in example


def test_node_langchain_integration_pins_minigraf_1_1_1():
    package_json = (
        ROOT / "integrations" / "langchain-js" / "package.json"
    ).read_text()

    assert '"minigraf": "1.1.1"' in package_json
    assert '"@langchain/core"' in package_json


def test_node_langchain_example_documents_expected_output():
    example = (
        ROOT / "integrations" / "langchain-js" / "minigraf-chat-history.mjs"
    ).read_text()

    assert "class MinigrafChatMessageHistory extends BaseChatMessageHistory" in example
    assert "Human: Remember that Minigraf stores agent memory." in example
    assert "AI: Got it. I will use Minigraf-backed chat history." in example
