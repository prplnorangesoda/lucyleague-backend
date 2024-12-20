CREATE TABLE IF NOT EXISTS users (
	id BIGSERIAL PRIMARY KEY,
	steamid VARCHAR(50) UNIQUE NOT NULL,
	username VARCHAR(50) NOT NULL,
	avatarurl VARCHAR(200) NOT NULL,
	permissions BIGINT DEFAULT 0 NOT NULL,
	created_at TIMESTAMPTZ NOT NULL,
	UNIQUE (steamid)
);
CREATE TABLE IF NOT EXISTS leagues (
	id BIGSERIAL PRIMARY KEY,
	name VARCHAR(50) NOT NULL,
	accepting_teams BOOLEAN DEFAULT FALSE NOT NULL,
	is_hidden BOOLEAN DEFAULT FALSE NOT NULL,
	created_at TIMESTAMPTZ NOT NULL
);
CREATE TABLE IF NOT EXISTS divisions (
	id BIGSERIAL PRIMARY KEY,
	leagueid BIGSERIAL NOT NULL,
	prio INT NOT NULL,
	name VARCHAR(50) NOT NULL,
	created_at TIMESTAMPTZ NOT NULL,
	CONSTRAINT FK_divisions_league FOREIGN KEY (leagueid) references leagues(id)
);
CREATE TABLE IF NOT EXISTS division_admins (
	id BIGSERIAL PRIMARY KEY,
	divisionid BIGSERIAL NOT NULL,
	userid BIGSERIAL NOT NULL,
	relation VARCHAR(50) NOT NULL DEFAULT 'Admin',
	CONSTRAINT FK_division_admins_user FOREIGN KEY (userid) references users(id),
	CONSTRAINT FK_division_admins_division FOREIGN KEY (divisionid) references divisions(id)
);
CREATE TABLE IF NOT EXISTS teams (
	id BIGSERIAL PRIMARY KEY,
	team_tag VARCHAR(6) NOT NULL DEFAULT 'Tag',
	team_name VARCHAR(200) NOT NULL DEFAULT 'Unnamed',
	owner_id BIGSERIAL NOT NULL,
	created_at TIMESTAMPTZ NOT NULL,
	CONSTRAINT FK_teams_owner_id FOREIGN KEY (owner_id) references users(id)
);
-- A team can be in multiple leagues at the same time, and have different rosters, while having the same (base) team.
CREATE TABLE IF NOT EXISTS teamDivAssociations (
	id BIGSERIAL PRIMARY KEY,
	roster_name VARCHAR(50),
	teamid BIGSERIAL NOT NULL,
	divisionid BIGSERIAL NOT NULL,
	points_up BIGINT NOT NULL DEFAULT 0,
	points_down BIGINT NOT NULL DEFAULT 0,
	created_at TIMESTAMPTZ NOT NULL,
	is_private BOOLEAN NOT NULL,
	CONSTRAINT FK_teamDivAssociation_team FOREIGN KEY (teamid) references teams(id),
	CONSTRAINT FK_teamDivAssociation_division FOREIGN KEY (divisionid) references divisions(id)
);
CREATE TABLE IF NOT EXISTS userTeamAssociation (
	id BIGSERIAL PRIMARY KEY,
	userid BIGSERIAL NOT NULL,
	teamdivid BIGSERIAL NOT NULL,
	created_at TIMESTAMPTZ NOT NULL,
	ended_at TIMESTAMPTZ,
	affiliation INT NOT NULL,
	CONSTRAINT FK_userTeamAssociation_teamdivid FOREIGN KEY (teamdivid) references teamDivAssociations(id),
	CONSTRAINT FK_userTeamAssociation_user FOREIGN KEY (userid) references users(id)
);
CREATE TABLE IF NOT EXISTS games (
	id BIGSERIAL PRIMARY KEY,
	title VARCHAR(50),
	leagueid BIGSERIAL NOT NULL,
	teamhomeid BIGSERIAL NOT NULL,
	teamawayid BIGSERIAL NOT NULL,
	created_at TIMESTAMPTZ NOT NULL,
	played_at TIMESTAMPTZ NOT NULL,
	CONSTRAINT FK_game_league FOREIGN KEY (leagueid) references leagues(id),
	CONSTRAINT FK_game_home FOREIGN KEY (teamhomeid) references teams(id),
	CONSTRAINT FK_game_away FOREIGN KEY (teamawayid) references teams(id)
);
CREATE TABLE IF NOT EXISTS authorizations (
	id BIGSERIAL PRIMARY KEY,
	userid BIGSERIAL NOT NULL,
	token TEXT NOT NULL,
	created_at TIMESTAMPTZ NOT NULL,
	expires TIMESTAMPTZ NOT NULL,
	CONSTRAINT FK_authorization_user FOREIGN KEY (userid) references users(id)
);
CREATE TABLE IF NOT EXISTS team_invites (
	id BIGSERIAL PRIMARY KEY,
	teamdivid BIGSERIAL NOT NULL,
	to_userid BIGSERIAL NOT NULL,
	from_userid BIGSERIAL NOT NULL,
	CONSTRAINT FK_team_invites_teamdivid FOREIGN KEY (teamdivid) references teamDivAssociations(id),
	CONSTRAINT FK_team_invites_to_userid FOREIGN KEY (to_userid) references users(id),
	CONSTRAINT FK_team_invites_from_userid FOREIGN KEY (from_userid) references users(id)
);
CREATE TABLE IF NOT EXISTS team_join_requests (
	leagueid BIGSERIAL NOT NULL,
	teamid BIGSERIAL NOT NULL,
	from_userid BIGSERIAL NOT NULL,
	CONSTRAINT team_join_requests_leagueid FOREIGN KEY (leagueid) references leagues(id),
	CONSTRAINT team_join_requests_teamid FOREIGN KEY (teamid) references teams(id),
	CONSTRAINT team_join_requests_from_userid FOREIGN KEY (from_userid) references users(id)
);