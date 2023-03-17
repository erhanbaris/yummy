import yummy

requests = []

def pre_deviceid_auth(model: yummy.DeviceIdAuth):
    pass

def post_deviceid_auth(model: yummy.DeviceIdAuth, success: bool):
    pass

def pre_email_auth(model: yummy.EmailAuth):
    pass

def post_email_auth(model: yummy.EmailAuth, success: bool):
    pass

def pre_customid_auth(model: yummy.CustomIdAuth):
    pass

def post_customid_auth(model: yummy.CustomIdAuth, success: bool):
    pass

def pre_logout(model: yummy.Logout):
    print(model.get_user_id() + " logout")

def post_logout(model: yummy.Logout, success: bool):
    pass

def pre_user_connected(model: yummy.Logout):
    print("pre_user_connected")

def post_user_connected(model: yummy.Logout, success: bool):
    print("post_user_connected")
