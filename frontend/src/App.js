import { useEffect, useState } from "react";
import './App.css';

function App() {
  const [messages, setMessages] = useState(["asd","asd","asd","asd","asd","asd","asd","asd","asd","asd","asd","asd"]);
  const [name, setName] = useState("");
  const [room, setRoom] = useState("");
  const [round, setRound] = useState(0);
  const [words, setWords] = useState(["apple","banana","pear", "apple","banana","pear", "apple","banana","pear","apple","banana","pear"]);
  const [yourWord, setYourWord] = useState("apple");
  const [declarationStarted, setDeclarationStarted] = useState(false);

  const [joined, setJoined] = useState(false); // new state
  const [players, setPlayers] = useState([]); // track players in room
  useEffect(() => {
    if (room) {
      document.title = `Castlefall Room: ${room}`;
    } else {
      document.title = "Castlefall";
    }
  }, [room]);

  // Connect to SSE (mocked for now)
  useEffect(() => {
    const events = new EventSource("http://127.0.0.1:3000/events");
    events.onmessage = (event) => {
      setMessages((prev) => [...prev, event.data]);

      // if the event is a join message, update players
      if (event.data.includes("joined room")) {
        const nameInMessage = event.data.split(" joined room")[0];
        setPlayers((prev) => {
          if (!prev.includes(nameInMessage)) return [...prev, nameInMessage];
          return prev;
        });
      }
    };
    return () => events.close();
  }, []);

  const joinRoom = async () => {
    console.log(`Mock join: name=${name}, room=${room}`);

    // simulate server response delay
    await new Promise((res) => setTimeout(res, 500));

    // pretend server broadcasts a message
    const joinMsg = `${name} joined room ${room} (mocked)`;
    setMessages((prev) => [...prev, joinMsg]);

    // add current user to players
    setPlayers((prev) => [...prev, name]);
    setRound(0);
    // hide the join form
    setJoined(true);
  };

  const startDeclaration = () => {
    console.log("Declaration started");
  };

 return (
    <div style={{ padding: "2rem" }}>
      
     
      {!joined ? (
        <div style={{ marginBottom: "2rem" }}>
          <input
            placeholder="Name"
            value={name}
            onChange={(e) => setName(e.target.value)}
            style={{ marginRight: "1rem" }}
          />
          <input
            placeholder="Room"
            value={room}
            onChange={(e) => setRoom(e.target.value)}
            style={{ marginRight: "1rem" }}
          />
          <button onClick={joinRoom}>Join</button>
        </div>
      ) : (
        <div>

           <h3 style={{ marginTop: "2rem" }}>Log</h3>
           <div className="messages-container">
            <ul>
              {messages.map((msg, i) => (
                <li key={i}>{msg}</li>
              ))}
            </ul>
          </div>

          <h3>Round {round}</h3>
          <h3>Players </h3>
            <div  className="words-grid">
            {players.map((player, i) => (
              <div
                key={i}
                className="word-item"
              >
                {player}
              </div>
            ))}
          </div>

          <h3>Words</h3>
          <div className="words-grid">
            {words.map((word, i) => (
              <div
                key={i}
                className="word-item"
              >
                {word}
              </div>
            ))}
          </div>

          <div>
            Your word is {yourWord}
          </div>

       <div
            style={{
              display: "flex",
              flexDirection: "column",
              gap: "1rem",
              alignItems: "flex-start" // aligns buttons to the left
            }}
          >
            {!declarationStarted && (
              <button onClick={startDeclaration} className="button-modern">
                Start Declaration
              </button>
            )}

            {!declarationStarted && (
              <button onClick={startDeclaration} className="button-modern">
                I won!
              </button>
            )}
          </div>


        </div>
      )}

    </div>
  );
}

export default App;
