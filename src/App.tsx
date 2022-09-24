import { invoke } from "@tauri-apps/api/tauri"
import { useEffect, useState } from "react";

// TODO
// constants for difficulties, sourceless question source_ids (anything else?)
// make TS types for data fetched from disk
// backend commands for getting all topics, sources
// --different data from disk --> questions, topics, sources, 
// design static parts of frontend
// eventually add users + signin page

function App() {
    const [a, setA] = useState<unknown>(null)
    useEffect(() => {
        const fetcher = async () => {
            const fetched = await invoke('get_questions', {
                options: {
                    user: 1,
                    diff: ["EASY", "HARD"],
                    topics: [1, 2, 3, 4, 5],
                    solved: [false],
                    source_ids: [0, 1, 2, 3],
                    starred: [true],
                    range: [[0, 100,]]
                }
            })
            setA(fetched)
        }

        fetcher()
    }, [])
    
    console.log(a)
  	return (
		<div className="App">
		</div>
	);
}

export default App;
