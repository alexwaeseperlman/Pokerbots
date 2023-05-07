import React from "react";
import { useTeam, useUser } from "./state";
import CreateTeam from "./CreateTeam";

export default function ManageTeam() {
  const user = useUser();
  const team = useTeam();

  /*
{{> include/head}}
{{ #if team }}
    {{! TEAM! }}
    Hi {{user.display_name}}. You are a member of team {{team.team_name}}.
    <br/>

    {{#if message}}
        <p>{{message}}</p>
    {{/if}}
    Members of "{{team.team_name}}":
    <br/>

    {{#each team.members}}
        {{this.display_name}}<br/>
    {{/each}}
    <br/>

    {{#if isOwner}}
        <div>
            Invite link (click to copy):
            <input type="text" id="invite-link" readonly />
            <button id="generate-invite">Generate new link</button>
        </div>

       <p> {{team.elo}}</p>

        <form action="/api/delete-team">
            <button type="submit" onclick="confirmDelete(event)">Delete Team</button>
        </form>

    {{else}}
        <form action="/api/leave-team">
            <input type="submit" value="Leave Team"/>
        </form>
    {{/if}}

    Upload bot:
    <form action="/api/upload-bot" enctype="multipart/form-data" method="POST">
        <input type="file" name="bot" accept=".zip"/>
        <button type="submit">Submit</button>
    </form>

{{ else }}
    {{ #if user}}
    <div>
        <div class="greetings"><span>Hi {{ user.display_name }}</span> &#x1F44B;</div>
        
        <p>You do not have a team. Create one below or join one by pasting an invitation link into your browser.</p>

        <form action="/api/create-team" onsubmit="return checkTeamName(document.getElementById('team-name-input').value)">

            {{#if message}}
                <p>{{message}}</p>
            {{/if}}

        <div class="email-container">
           <input type="text" class="team-name" id="team-name-input" name="team_name" placeholder="Team Name"/>
           <button type="submit" value="Create Team" class="submit-name-button">Submit</button>
        </div>

        </form>
        <button class="random-button" onclick="generator()"> &#x1F3B2; Random</button>
        </div>

    {{ else }}
        {{#if message}}
            <p>{{message}}</p>
        {{/if}}
        {{> login}}
    {{/if}}
{{/if}}

<script>
    function confirmDelete(event) {
        if (confirm("Are you sure you want to delete your team? This action cannot be undone.")) {
            // Proceed with form submission
        } else {
            // Prevent form submission
            event.preventDefault();
        }
    }
</script>

<script> // checking for team name

    function checkTeamName(input) {
    // List of banned words
    const bannedWords = ["badword1", "badword2", "badword3"];

    // Split input into words
    const words = input.split(" ");

    // Check input against each banned word
    for (let i = 0; i < words.length; i++) {
        if (bannedWords.includes(words[i].toLowerCase())) {
        alert("Your input contains a banned word. Please try again.");
        return false;
        }
    }

    // Check input length
    if (input.length < 3) {
        alert("Team name should be at least 3 characters long. Please try again.");
        return false;
    }
    if (input.length > 20) {
        alert("Team name should be at most 20 characters long. Please try again.");
        return false;
    }

    return true;
    }

</script>

<script>
    // generate invite links if necessary
    const inviteLink = document.getElementById('invite-link');
    const genInviteButton = document.getElementById('generate-invite');
    let lastGenerated = parseInt(localStorage.getItem('lastGenerated'));

const genInvite = () => {
    if (lastGenerated !== null && (Date.now() - lastGenerated) < 10000) {
        inviteLink.value = localStorage.getItem('lastLink');
        return;
    }

    // Generate a new invite link
    fetch("/api/make-invite")
        .then(async (result) => {
            const newInviteLinkValue = window.location.origin + '/api/join-team?invite_code='+(await result.text());
            inviteLink.value = newInviteLinkValue;
            lastGenerated = Date.now();
            localStorage.setItem('lastGenerated', lastGenerated.toString());
            localStorage.setItem('lastLink', newInviteLinkValue);
        });
}
    window.addEventListener('load', () => {
        genInvite()
        genInviteButton.addEventListener('click', genInvite);
        inviteLink.addEventListener('click', () => {
            inviteLink.select()
            inviteLink.setSelectionRange(0, 1e5);
            navigator.clipboard.writeText(inviteLink.value)
        })
    });
</script>
{{> include/foot}}
*/

  if (team && user) {
    return (
      <>
        Hi {user.display_name}. You are a member of team {team.team_name}.
        <br />
        Members of "{team.team_name}":
        <br />
        {team.members.map((member) => (
          <>{member.display_name}</>
        ))}
      </>
    );
  } else if (user) {
    return <CreateTeam />;
  } else {
    return <></>;
  }
}
