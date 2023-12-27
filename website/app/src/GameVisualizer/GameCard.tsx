// GameCard.tsx
import React from 'react';
import './CardStyles.css'; // Import the CSS file


interface CardProps {
  card: {
    rank: string;
    suit: string;
  };
}

const GameCard: React.FC<CardProps> = ({ card }) => {
  const getSuitSymbol = (suit: string) => {
    switch (suit) {
      case 'hearts':
        return '♥';
      case 'diamonds':
        return '♦';
      case 'clubs':
        return '♣';
      case 'spades':
        return '♠';
      default:
        return '';
    }
  };

  return (
    <div className={`card ${card.suit}`}>
      <div className="card-content">
        <div className="suit top left">{getSuitSymbol(card.suit)}</div>
        <div className="suit top right">{getSuitSymbol(card.suit)}</div>
        <div className="card-center">
          <span className="rank">{card.rank}</span>
        </div>
        <div className="suit-down">
          <div className="suit bottom left">{getSuitSymbol(card.suit)}</div>
          <div className="suit bottom right">{getSuitSymbol(card.suit)}</div>
        </div>
      </div>
    </div>
  );
};

export default GameCard;

