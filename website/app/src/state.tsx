import { atom, useAtom, useAtomValue, useSetAtom, WritableAtom } from "jotai";
import { atomFamily } from "jotai/utils";
import { useEffect } from "react";
import { matchPath } from "react-router-dom";

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
  const [team, fetchTeam] = useMyTeam();
  const fetchUser = async () => {
    const data: User = (await (
      await fetch(`${apiUrl}/my-account`)
    ).json()) as unknown as User;
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
  invites?: string[];
  owner: string;
  score: number | null;
  active_bot?: number;
};
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
  if (games.length == 0) return [] as Game[];
  // replace team ids with their objects
  const botIds = new Set<number>([]);
  for (const game of games) botIds.add(game.bot_a), botIds.add(game.bot_b);
  const bots = await fetch(`${apiUrl}/bots?ids=${[...botIds].join(",")}`).then(
    (res) => res.json()
  );

  const teamIds = new Set<number>([]);
  for (const bot of bots) teamIds.add(bot.team);
  const teams = await fetch(
    `${apiUrl}/teams?ids=${[...teamIds].join(",")}`
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

const myTeamAtom = atom<Promise<Team | null>>(
  fetch(`${apiUrl}/my-team`).then((res) => res.json())
);

export const useMyTeam = () => {
  const [team, setTeam] = useAtom(myTeamAtom);
  const fetchTeam = () => {
    setTeam(fetch(`${apiUrl}/my-team`).then((res) => res.json()));
  };
  return [team, fetchTeam] as const;
};

let pathTeam = matchPath("/team/:id", window.location.pathname)?.params.id;
let selectedTeam;
if (pathTeam) selectedTeam = parseInt(pathTeam);
else selectedTeam = null;
const selectedTeamAtom = atom<number | null>(selectedTeam);
// choose default value based on route
const teamAtom = atom<Promise<Team | null>>(
  pathTeam
    ? fetch(`${apiUrl}/teams?ids=${selectedTeam ?? ""}&fill_members=true`)
        .then((res) => res.json())
        .then((teams) => teams[0])
    : fetch(`${apiUrl}/my-team`).then((res) => res.json())
);

export const useTeam = () => {
  const [selectedTeam, setSelectedTeam] = useAtom(selectedTeamAtom);
  const [team, setTeam] = useAtom(teamAtom);
  const fetchTeam = () => {
    if (!selectedTeam)
      return setTeam(fetch(`${apiUrl}/my-team`).then((res) => res.json()));
    else {
      setTeam(
        fetch(`${apiUrl}/teams?ids=${selectedTeam}&fill_members=true`)
          .then((res) => res.json())
          .then((teams) => teams[0])
      );
    }
  };
  useEffect(() => {
    fetchTeam();
  }, [selectedTeam]);
  return [team, fetchTeam, setSelectedTeam] as const;
};
