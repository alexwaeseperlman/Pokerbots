var adjective = ["Stoic", "Anxious", "Overweight", "Demonic", "Agile", "Misunderstood", "Majestic", "Clever","Wild", "Lucky", "Mischievous", "Jealous", "Lazy","Sneaky", "Bored", 
"Annoying", "Demonic", "Bizarre", "Quirky", "Naive", "Religious", "Silent", "Quiet", "Mute", "Noisy", "Hilarious", "Indebted",
 "Thick", "Amusing", "Eccentric", "Straight", "Broken", "Highly Trained", "Skilled", "Boring","Psychopathic", "Funky", "Chill", "Drugged", "Ruthless", "Merciless", 
 "Canadian", "American", "Asian", "European", "African", "Broke", "Crying", "Sophisticated", "Efficient", "Soft", "Childish", "Straight A", "Unemployed", 
 "Educated"] 
var object = ["Taco", "Burritos", "Milkshakes", "ex-FAANGs", "OS", "Sharks", "Donkeys", "Monkeys", "Quesadillas", "Dealers", "Titans", "Dragons", "Raccoons", "Alpacas", "Llamas", "Jokers", "Quants", "Zillionaires", "Hustlers", "Owls", "Foxes", "Panthers", "Squad", "Squad", "AGI", "Compilers", "Bluffers", "Prodigies", "Pals", "Crew", "Monks", "Ninjas", "Cobras", "Gorillaz", "Peacocks", "Falcons", "Rabbits", "Swans", "Gamblers", "Turtles", "Penguins", "Cheetah", "Traders", "Gods", "Coders", "Rats", "FAANG Interns", "GPUs",
"Know-It-Alls", "Sages", "Masters", "Wizards", "PhDs", "Undergrads", "Scholars", "Apes", "Monsters", "Pandas"]
var list;

function generator() {
    var name = "";
    do {
      name = adjective[Math.floor(Math.random() * adjective.length)] + " " + object[Math.floor(Math.random() * object.length)];
    } while (name.length > 20);
    document.getElementById("team-name-input").value = name;
  }
