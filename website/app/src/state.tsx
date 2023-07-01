import {
  Atom,
  atom,
  PrimitiveAtom,
  useAtom,
  useAtomValue,
  useSetAtom,
  WritableAtom,
} from "jotai";
import { atomFamily, atomWithStorage } from "jotai/utils";
import { useEffect } from "react";
import { matchPath } from "react-router-dom";

export const apiUrl = window.location.origin + "/api";
export type User = {
  email: string;
  display_name: string;
};
const userAtom = atomWithStorage<Promise<User | null>>(
  "user",
  fetch(`${apiUrl}/my-account`)
    .then((res) => res.json())
    .catch(() => null)
);

export const useUser = () => {
  const [user, setUser] = useAtom(userAtom);
  const [team, fetchTeam] = useMyTeam();
  const fetchUser = async () => {
    setUser(Promise.resolve(null));
    setUser(
      fetch(`${apiUrl}/my-account`)
        .then((res) => res.json())
        .catch(() => null)
    );
  };
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

export type Bot = {
  id: number;
  name: string;
  team: Team;
  uploaded_by: string;
  date_uploaded: number;
  build_status: number;
};

export type Game = {
  id: string;
  bot_a: Bot;
  bot_b: Bot;
  score_change: number;
  time: number;
  error_type: string | null;
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
  const teams = await fetch(`${apiUrl}/teams?ids=${[...teamIds].join(",")}`)
    .then((res) => res.json())
    .catch(() => []);

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

const myTeamAtom = atom<Promise<Team | null>>(Promise.resolve(null));

export const useMyTeam = () => {
  const [team, setTeam] = useAtom(myTeamAtom);
  const fetchTeam = () => {
    setTeam(
      fetch(`${apiUrl}/my-team`)
        .then((res) => res.json())
        .catch(() => null)
    );
  };

  useEffect(() => {
    fetchTeam();
  }, []);

  return [team, fetchTeam] as const;
};

// choose default value based on route
const teamAtom = atomFamily<string | null, PrimitiveAtom<Promise<Team | null>>>(
  (param) =>
    atomWithStorage(
      param ?? "my-team",
      param
        ? fetch(`${apiUrl}/teams?ids=${param ?? ""}&fill_members=true`)
            .then((res) => res.json())
            .then((teams) => teams[0])
            .catch(() => null)
        : fetch(`${apiUrl}/my-team`)
            .then((res) => res.json())
            .catch(() => null)
    )
);

export const useTeam = (selectedTeam: string | null) => {
  const [team, setTeam] = useAtom(teamAtom(selectedTeam));
  const fetchTeam = () => {
    setTeam(Promise.resolve(null));
    if (!selectedTeam)
      setTeam(
        fetch(`${apiUrl}/my-team`)
          .then((res) => res.json())
          .catch(() => null)
      );
    else {
      setTeam(
        fetch(`${apiUrl}/teams?ids=${selectedTeam}&fill_members=true`)
          .then((res) => res.json())
          .then((teams) => teams[0])
          .catch(() => null)
      );
    }
  };
  return [team, fetchTeam] as const;
};
