// GameCard.tsx
import React from "react";
import "./CardStyles.css"; // Import the CSS file
import { Card } from "@bindings/Card";

interface CardProps {
  card?: Card;
}

function valueToRank(value: number) {
  switch (value) {
    case 1:
      return "A";
    case 11:
      return "J";
    case 12:
      return "Q";
    case 13:
      return "K";
    default:
      return value;
  }
}

function GameCard({ card }: CardProps) {
  const getSuitSymbol = (suit: string) => {
    switch (suit) {
      case "Hearts":
        return "♥";
      case "Diamonds":
        return "♦";
      case "Clubs":
        return "♣";
      case "Spades":
        return "♠";
      default:
        return "";
    }
  };

  if (card) {
    console.log("card", card);
    return (
      <div className={`card ${card.suite.toLowerCase()}`}>
        <div className="card-content">
          <div className="suit top left">{getSuitSymbol(card.suite)}</div>
          <div className="suit top right">{getSuitSymbol(card.suite)}</div>
          <div className="card-center">
            <span className="rank">{valueToRank(card.value)}</span>
          </div>
          <div className="suit-down">
            <div className="suit bottom left">{getSuitSymbol(card.suite)}</div>
            <div className="suit bottom right">{getSuitSymbol(card.suite)}</div>
          </div>
        </div>
      </div>
    );
  } else {
    return (
      <div className={`card`}>
        <div className="card-content">
          <div className="card-center">
            <span className="rank"></span>
          </div>
        </div>
      </div>
    );
  }
}

export default GameCard;
