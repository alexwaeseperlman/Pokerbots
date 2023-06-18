while True:
    try:
        s = input()
    except EOFError:
        break

    if s == 'ping':
        print('pong')
    else:
        print('no')
