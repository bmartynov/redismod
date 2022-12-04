import pprint

import xid
import redis

if __name__ == '__main__':
    client = redis.Redis(host='localhost', port=6379)

    task_id = xid.Xid().string()
    # task_id = 'cdu75v324te69lj24teg'

    pprint.pprint(task_id)

    result = client.execute_command('task.create', task_id, "task::test", "1234567890", 3000, '{"id": 1000}')
    pprint.pprint(result)

    # result = client.execute_command('task.fail', task_id)
    # pprint.pprint(result)

    result = client.execute_command('task.info', task_id)
    pprint.pprint(result)
