# Sample bot

Here is what a sample bot looks like (both files are then zipped into a bot.zip):

- `bot.json`
    ```json
    {
        "name": "Raise with pocket pairs",
        "build": "g++ main.cpp -o main",
        "run": "./main"
    }
    ```

- `main.cpp`
    ```cpp
    #include <iostream>
    using namespace std;

    int vals[4] = {2, 5, 6, 7};
    int inp[5] = {0};
    int main() {
    int round = -1, position = 0;
    bool pocket = false;
    while (true) {
        char type;
        cin >> type;
        if (type == 'E') {
        round = -1;
        pocket = false;
        cerr << "Round ended." << endl << endl;
        } else if (type == 'P') {
        cin >> position;
        cerr << "My position is " << position << endl;
        } else if (type == 'C') {
        round++;
        char a;
        cerr << "Cards: ";
        char val;
        for (int i = 0; i < vals[round]; i++) {
            char nowVal;
            cin >> nowVal;
            cerr << nowVal;
            cin >> a;
            if (i == 1)
            pocket = (nowVal == val);
            val = nowVal;
            cerr << a << ' ';
        }
        cerr << endl;
        } else {
        cerr << "Stacks: ";
        for (int i = 0; i < 5; i++) {
            cin >> inp[i];
        }
        for (int i = 0; i < 5; i++) {
            cerr << inp[i] << " ";
        }
        cerr << endl;

        cerr << "Deciding on an action: ";
        if (pocket && inp[position + 1] <= inp[position + 3] - 5) {
            cerr << "raising for pocket pairs" << endl;
            cout << "R5" << endl;
        } else if (inp[position + 1] == inp[0]) {
            cerr << "Check." << endl;
            cout << 'X' << endl;
        } else {
            cerr << "Call." << endl;
            cout << 'C' << endl;
        }
        cout.flush();
        }
        // cerr << endl;
    }
    }
    ```