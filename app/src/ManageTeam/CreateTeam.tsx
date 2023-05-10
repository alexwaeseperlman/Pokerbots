import React, { useState } from "react";
import { useUser } from "../state";
import randomTeamName from "./random-name";

export default function CreateTeam() {
  const user = useUser();
  const [teamName, setTeamName] = useState("");
  return (
    <>
      <div className="greetings">
        <span>Hi {user?.display_name}</span> {"👋"}
      </div>
      <p>
        You do not have a team. Create one below or join one by pasting an
        invitation link into your browser.
      </p>
      <form action="/api/create-team">
        <div className="email-container">
          <input
            value={teamName}
            type="text"
            className="team-name"
            id="team-name-input"
            name="team_name"
            placeholder="Team Name"
            onChange={(e) => setTeamName(e.target.value)}
          />
          <button
            type="submit"
            value="Create Team"
            className="submit-name-button"
          >
            Submit
          </button>
        </div>
      </form>
      <button
        className="random-button"
        onClick={() => setTeamName(randomTeamName())}
      >
        {" "}
        &#x1F3B2; Random
      </button>
    </>
  );
}
