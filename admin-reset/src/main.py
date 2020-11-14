import base64
import postgresql

admin_reset = base64.b64decode("{{admin_reset.sql}}").decode('utf-8')

db = postgresql.open("pq://{{USERNAME}}:{{PASSWORD}}@localhost/{{DBNAME}}")
db.execute(admin_reset)
db.close()
