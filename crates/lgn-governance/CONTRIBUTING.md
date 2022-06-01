# Contributing to the governance service

## Required tools

Developing on the governance service requires the following tools:

- `sqlx-cli`: A database client for Rust's `sqlx` crate. You may install it with: `cargo install sqlx-cli`
- `mysql-client`: A MySQL client. You may install it with: `sudo apt-get install mysql-client`

## Running a local MySQL instance for testing

The governance service requires a MySQL instance to be running properly. The
easiest way to achieve that in development is to run a local instance of MySQL
through Docker:

```bash
docker run --name governance-mysql -e MYSQL_ROOT_PASSWORD=pw -d -p 3306:3306 mysql:latest
```

Once it runs, you still have to create the database (which the service will not
do for you):

```bash
export DATABASE_URL=mysql://root:pw@localhost:3306/lgn_governance
sqlx database setup
```

**Note: setting the `DATABASE_URL` environment variable is a very convenient way
to ease the process, as the Governance service server binary will automatically
use it.**

For some debugging scenarii, you may want to connect to the database using the
following command:

```bash
mysql -h localhost -u root --protocol=TCP --password
```

**Note: When prompted, enter the password you set for the root user (which should be
`pw`, if you copy-pasted the `docker run` line above).**

Giving an exhaustive list of the commands you can run on the database is not in
the scope of this guide, but here are nevertheless some of the most common ones
to get you started:

### Listing databases

```sql
show databases;
```

### Using a specific database

```sql
use lgn_governance;
```

### Listing tables

```sql
show tables;
```

### Listing the first 10 rows of a table

```sql
select * from table_name limit 10;
```