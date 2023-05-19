import { atom, useAtom, useAtomValue, useSetAtom, WritableAtom } from "jotai";
import { atomFamily } from "jotai/utils";
import { useEffect } from "react";

export type User = {
  email: string;
  display_name: string;
};
const userAtom = atom<User | null | undefined>(
  (JSON.parse(localStorage.getItem("user") || "null") ?? undefined) as
    | User
    | null
    | undefined
);

export const useUser = () => {
  const [user, setUser] = useAtom(userAtom);
  const [team, fetchTeam] = useTeam();
  const fetchUser = async () => {
    const data: User = (await (
      await fetch(`${apiUrl}/my-account`)
    ).json()) as unknown as User;
    console.log(data);
    localStorage.setItem("user", JSON.stringify(data));
    setUser(data);
    fetchTeam();
  };
  // fetch user
  useEffect(() => {
    fetchUser();
  }, []);
  return [user, fetchUser] as const;
};

export type Team = {
  id: number;
  team_name: string;
  members: User[];
  invites: string[];
  owner: string;
  score: number | null;
};

const teamAtom = atom<Team | null | undefined>(
  (JSON.parse(localStorage.getItem("team") || "null") ?? undefined) as
    | Team
    | null
    | undefined
);

export const useTeam = () => {
  const [team, setTeam] = useAtom(teamAtom);
  const fetchTeam = async () => {
    const data: Team = (await (
      await fetch(`${apiUrl}/my-team`)
    ).json()) as unknown as Team;
    setTeam(data);
    localStorage.setItem("team", JSON.stringify(data));
  };
  // fetch team
  useEffect(() => {
    fetchTeam();
  }, []);
  return [team, fetchTeam] as const;
};

const teamsAtom = atomFamily((id) =>
  atom(
    async () =>
      fetch(`${apiUrl}/teams?id=${id}`).then((res) =>
        res.json()
      ) as unknown as Team[]
  )
);

console.log(import.meta.env.APP_PFP_ENDPOINT);
export const pfpEndpoint = import.meta.env.APP_PFP_ENDPOINT;
export const apiUrl = window.location.origin + "/api";

export type Bot = {
  id: number;
  name: string;
  team: Team;
  uploaded_by: string;
  date_uploaded: number;
};

export type Game = {
  id: string;
  bot_a: Bot;
  bot_b: Bot;
  score_change: number;
  time: number;
};

// take a list of games that have bot ids and replace them with bot objects
export async function fillInGames(
  games: ({ bot_a: number; bot_b: number } & Omit<Game, "bot_a" | "bot_b">)[]
) {
  // replace team ids with their objects
  const botIds = new Set<number>([]);
  for (const game of games) botIds.add(game.bot_a), botIds.add(game.bot_b);
  const bots = await fetch(`${apiUrl}/bots?id=${[...botIds].join(",")}`).then(
    (res) => res.json()
  );

  const teamIds = new Set<number>([]);
  for (const bot of bots) teamIds.add(bot.team);
  const teams = await fetch(
    `${apiUrl}/teams?id=${[...teamIds].join(",")}`
  ).then((res) => res.json());

  const teamMap = new Map(teams.map((team) => [team.id, team]));
  const botMap = new Map(
    bots.map((bot) => [bot.id, { ...bot, team: teamMap.get(bot.team) }])
  );

  return games.map((game) => ({
    ...game,
    bot_a: botMap.get(game.bot_a) as Bot,
    bot_b: botMap.get(game.bot_b) as Bot,
  }));
}
