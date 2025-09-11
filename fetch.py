import requests
import sqlite3
import time

def setup_db():
    conn = sqlite3.connect('woogles.db')
    conn.execute('''
        CREATE TABLE IF NOT EXISTS games (
            game_id TEXT PRIMARY KEY,
            winner INTEGER,
            gcg TEXT,
            player1 TEXT,
            player2 TEXT
        )
    ''')
    conn.commit()
    return conn

def get_games(username, offset=0):
    url = 'https://woogles.io/api/game_service.GameMetadataService/GetRecentGames'
    data = {"username": username, "numGames": 1000, "offset": offset}
    r = requests.post(url, json=data, headers={'Content-Type': 'application/json'})
    return r.json().get('game_info', [])

def get_gcg(game_id):
    url = 'https://woogles.io/api/game_service.GameMetadataService/GetGCG'
    r = requests.post(url, json={"game_id": game_id}, headers={'Content-Type': 'application/json'})
    return r.json().get('gcg', '')

def save_game(conn, game_id, winner, gcg, player1, player2):
    conn.execute('''
        INSERT OR IGNORE INTO games (game_id, winner, gcg, player1, player2)
        VALUES (?, ?, ?, ?, ?)
    ''', (game_id, winner, gcg, player1, player2))

def fetch_and_save(username="HastyBot"):
    conn = setup_db()
    offset = 0
    
    while True:
        print(f"\nFETCHING GAMES FOR {username}, offset {offset}")
        try:
            games = get_games(username, offset)
        except Exception as e:
            print(f"Failed fetching at: {offset}: {e}")
            offset += 1000
            continue
        
        if not games:
            print("No more games")
            break
            
        for game in games:
            game_id = game['game_id']
            
            # only care about these for now
            # - CSW24 + NWL23
            # - standard end of game, no resign or disconnects, etc
            # - std board layout
            lexicon = game.get('game_request', {}).get('lexicon')
            if (lexicon not in ['CSW24', 'NWL23'] or 
                game.get('game_request', {}).get('rules', {}).get('board_layout_name') != 'CrosswordGame' or
                game.get('game_end_reason') != 'STANDARD'):
                continue
            
            exists = conn.execute('SELECT 1 FROM games WHERE game_id = ?', (game_id,)).fetchone()
            if exists:
                print(f"{game_id} already exists")
                continue
                
            winner = game['winner']
            player1 = game['players'][0]['nickname']
            player2 = game['players'][1]['nickname']
            
            print(f"Fetching {game_id}: {player1} vs {player2}, winner: {winner}")
            gcg = get_gcg(game_id)
            
            save_game(conn, game_id, winner, gcg, player1, player2)
            conn.commit()
            
            time.sleep(0.05)
            
        offset += len(games)
    
    conn.close()

fetch_and_save()