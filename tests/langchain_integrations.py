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


def test_graphrag_integration_pins_dependencies():
    requirements = (
        ROOT / "integrations" / "graphrag-python" / "requirements.txt"
    ).read_text()

    assert "minigraf==1.1.1" in requirements
    assert "chromadb==1.5.9" in requirements


def test_graphrag_example_documents_expected_output():
    example = (
        ROOT / "integrations" / "graphrag-python" / "graphrag_minigraf.py"
    ).read_text()

    assert "def main()" in example
    assert 'Query: "storing time-varying relationships"' in example
    assert "Match: temporal-data" in example
    assert "Related: graph-db, knowledge-graph" in example


def test_llamaindex_integration_pins_dependencies():
    requirements = (
        ROOT / "integrations" / "llamaindex-python" / "requirements.txt"
    ).read_text()

    assert "minigraf==1.1.1" in requirements
    assert "llama-index-core" in requirements


def test_llamaindex_example_documents_expected_output():
    example = (
        ROOT / "integrations" / "llamaindex-python" / "minigraf_graph_store.py"
    ).read_text()

    assert "class MinigrafGraphStore(SimpleGraphStore)" in example
    assert "myapp depends-on: pydantic==2.0, requests==2.28" in example
    assert "requests depends-on: urllib3==1.26" in example
    assert "myapp at tx 3 depended-on: pydantic==1.10, requests==2.28" in example
